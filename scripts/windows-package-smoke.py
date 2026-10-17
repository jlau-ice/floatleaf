"""Run against an ordinary packaged EXE on an interactive Windows desktop.

Uses only synthetic shortcut input and native HWND queries. No debug IPC,
novels, user settings changes, or third-party automation dependencies.
"""
import argparse
import ctypes as c
from ctypes import wintypes as w
import itertools
import json
from pathlib import Path
import platform
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("exe", type=Path)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--expect-stuck", action="store_true")
parser.add_argument("--screenshot", type=Path)
args = parser.parse_args()

u = c.WinDLL("user32", use_last_error=True)
enum_proc = c.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
u.EnumWindows.argtypes = [enum_proc, w.LPARAM]
u.GetWindowThreadProcessId.argtypes = [w.HWND, c.POINTER(w.DWORD)]
u.GetWindowTextW.argtypes = [w.HWND, w.LPWSTR, c.c_int]
u.IsWindowVisible.argtypes = [w.HWND]
u.GetForegroundWindow.restype = w.HWND
u.SetForegroundWindow.argtypes = [w.HWND]
u.GetWindowLongW.argtypes = [w.HWND, c.c_int]
u.ShowWindow.argtypes = [w.HWND, c.c_int]
u.IsIconic.argtypes = [w.HWND]
u.IsZoomed.argtypes = [w.HWND]
u.GetWindowRect.argtypes = [w.HWND, c.POINTER(w.RECT)]
u.SendMessageW.argtypes = [w.HWND, w.UINT, w.WPARAM, w.LPARAM]
u.SetProcessDPIAware()
u.DestroyWindow.argtypes = [w.HWND]
u.CreateWindowExW.argtypes = [w.DWORD, w.LPCWSTR, w.LPCWSTR, w.DWORD,
                            c.c_int, c.c_int, c.c_int, c.c_int,
                            w.HWND, w.HMENU, w.HINSTANCE, w.LPVOID]
u.CreateWindowExW.restype = w.HWND


class Keyboard(c.Structure):
    _fields_ = [("vk", w.WORD), ("scan", w.WORD), ("flags", w.DWORD),
                ("time", w.DWORD), ("extra", c.c_size_t)]


class Mouse(c.Structure):
    _fields_ = [("dx", w.LONG), ("dy", w.LONG), ("data", w.DWORD),
                ("flags", w.DWORD), ("time", w.DWORD), ("extra", c.c_size_t)]


class Payload(c.Union):
    _fields_ = [("keyboard", Keyboard), ("mouse", Mouse)]


class Input(c.Structure):
    _fields_ = [("type", w.DWORD), ("payload", Payload)]


u.SendInput.argtypes = [w.UINT, c.POINTER(Input), c.c_int]
CTRL, ALT, SPACE, ESC = 0xA2, 0xA4, 0x20, 0x1B


def wait(seconds=0.2):
    until = time.monotonic() + seconds
    msg = w.MSG()
    while time.monotonic() < until:
        while u.PeekMessageW(c.byref(msg), None, 0, 0, 1):
            u.TranslateMessage(c.byref(msg))
            u.DispatchMessageW(c.byref(msg))
        time.sleep(0.01)


def key(vk, down=True):
    event = Input(type=1, payload=Payload(keyboard=Keyboard(vk=vk, flags=0 if down else 2)))
    if u.SendInput(1, c.byref(event), c.sizeof(Input)) != 1:
        raise RuntimeError("SendInput failed; an unlocked interactive desktop is required")


def release_all():
    for vk in [SPACE, ESC, ALT, CTRL]:
        key(vk, False)
    wait()


def press_display():
    for vk in [CTRL, ALT, SPACE]:
        key(vk)
    wait()


def find_window(pid, name):
    found = []

    @enum_proc
    def visit(hwnd, _):
        owner = w.DWORD()
        u.GetWindowThreadProcessId(hwnd, c.byref(owner))
        title = c.create_unicode_buffer(256)
        u.GetWindowTextW(hwnd, title, len(title))
        if owner.value == pid and title.value == name:
            found.append(hwnd)
        return True

    u.EnumWindows(visit, 0)
    return found[0] if found else None


checks = []


def check(name, value):
    checks.append({"name": name, "passed": bool(value)})


def check_settings(main):
    dwm = c.WinDLL("dwmapi")
    dwm.DwmGetWindowAttribute.argtypes = [w.HWND, w.DWORD, w.LPVOID, w.DWORD]
    for name, attribute, expected in [("caption", 35, 0xE9F2F4),
                                       ("text", 36, 0x323B20), ("light-controls", 20, 0)]:
        value = w.DWORD()
        result = dwm.DwmGetWindowAttribute(main, attribute, c.byref(value), c.sizeof(value))
        check(f"settings-{name}", result == 0 and value.value == expected)
    style = u.GetWindowLongW(main, -16)
    check("settings-native-caption-resize-min-max", style & 0x00CF0000 == 0x00CF0000)
    u.ShowWindow(main, 6)
    wait()
    check("settings-minimize", u.IsIconic(main))
    u.ShowWindow(main, 3)
    wait()
    check("settings-maximize", u.IsZoomed(main))
    u.ShowWindow(main, 9)
    wait()
    check("settings-restore", not u.IsZoomed(main) and not u.IsIconic(main))
    if args.screenshot:
        u.SetForegroundWindow(main)
        wait(1)
        rect = w.RECT()
        u.GetWindowRect(main, c.byref(rect))
        args.screenshot.parent.mkdir(parents=True, exist_ok=True)
        path = str(args.screenshot.resolve()).replace("'", "''")
        subprocess.run(["powershell", "-NoProfile", "-Command", f"""
            Add-Type -AssemblyName System.Drawing
            $image = [Drawing.Bitmap]::new({rect.right - rect.left}, {rect.bottom - rect.top})
            $graphics = [Drawing.Graphics]::FromImage($image)
            $graphics.CopyFromScreen({rect.left}, {rect.top}, 0, 0, $image.Size)
            $image.Save('{path}', [Drawing.Imaging.ImageFormat]::Png)
            $graphics.Dispose()
            $image.Dispose()
        """], check=True)


app = subprocess.Popen([str(args.exe.resolve())])
fixture = None
try:
    for _ in range(300):
        overlay = find_window(app.pid, "FloatLeaf 阅读")
        if overlay:
            break
        if app.poll() is not None:
            raise RuntimeError(f"App exited before creating its overlay: {app.returncode}")
        wait(0.1)
    else:
        raise RuntimeError("Overlay HWND not found")
    wait(2)
    main = find_window(app.pid, "浮页 FloatLeaf")
    if not args.expect_stuck:
        if not main:
            raise RuntimeError("Settings HWND not found")
        check_settings(main)
    fixture = u.CreateWindowExW(0, "STATIC", "FloatLeaf regression input fixture",
                                0x00CF0000, 50, 50, 640, 480, None, None, None, None)
    u.ShowWindow(fixture, 5)
    u.SetForegroundWindow(fixture)
    wait()
    if u.GetForegroundWindow() != fixture:
        raise RuntimeError("Input fixture could not become foreground")
    check("initial-hidden", not u.IsWindowVisible(overlay))

    for order in itertools.permutations([CTRL, ALT, SPACE]):
        release_all()
        press_display()
        check(f"show-{order}", u.IsWindowVisible(overlay))
        check(f"show-no-focus-{order}", u.GetForegroundWindow() == fixture)
        styles = u.GetWindowLongW(overlay, -20)
        check(f"noactivate-topmost-clickthrough-{order}", styles & 0x08000028 == 0x08000028)
        key(order[0], False)
        wait()
        check(f"release-hides-{order}", not u.IsWindowVisible(overlay))
        for vk in order[1:]:
            key(vk, False)
        wait()
        check(f"release-no-focus-{order}", u.GetForegroundWindow() == fixture)

    release_all()
    press_display()
    key(ESC)
    wait()
    check("emergency-hides", not u.IsWindowVisible(overlay))
    key(ESC, False)
    key(SPACE, False)
    wait()
    key(SPACE)  # Old modifiers are still down; this must not reveal content.
    wait()
    check("emergency-blocks-old-chord", not u.IsWindowVisible(overlay))
    release_all()
    press_display()
    check("fresh-chord-recovers", u.IsWindowVisible(overlay))
    release_all()
    check("fresh-chord-release-hides", not u.IsWindowVisible(overlay))

    if not args.expect_stuck:
        u.SendMessageW(main, 0x0010, 0, 0)  # WM_CLOSE must hide settings, not exit the reader.
        wait()
        check("settings-close-hides-keeps-app", not u.IsWindowVisible(main) and app.poll() is None)

    failures = [item["name"] for item in checks if not item["passed"]]
    reproduced = "emergency-hides" in failures and any(x.startswith("release-hides-") for x in failures)
    passed = reproduced if args.expect_stuck else not failures
    report = {"os": platform.platform(), "exe": args.exe.name,
              "expectedOldBug": args.expect_stuck, "passed": passed, "checks": checks,
              "scope": "Synthetic system shortcuts + HWND visibility/focus/styles; not physical keyboard or pixel tests"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    if not passed:
        raise SystemExit(1)
finally:
    release_all()
    if fixture:
        u.DestroyWindow(fixture)
    if app.poll() is None:
        app.terminate()
        app.wait(timeout=10)
