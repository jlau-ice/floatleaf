#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>
#include <wtsapi32.h>
#include <stdbool.h>
#include <commdlg.h>
#include <stdlib.h>
#include <dwmapi.h>
#include <stdint.h>
#include <wchar.h>

// Match the settings page while keeping the native caption and window controls.
long fl_style_settings(void *raw) {
    COLORREF background = RGB(244, 242, 233); // App.vue #f4f2e9
    COLORREF foreground = RGB(32, 59, 50);    // App.vue #203b32
    HRESULT result = DwmSetWindowAttribute((HWND)raw, DWMWA_CAPTION_COLOR,
                                          &background, sizeof(background));
    if (FAILED(result)) return result;
    return DwmSetWindowAttribute((HWND)raw, DWMWA_TEXT_COLOR,
                                 &foreground, sizeof(foreground));
}

static void (*hide_callback)(void);
static void (*displays_callback)(void);
static DWORD last_show_error;
void fl_show(void *raw) {
    HWND window = (HWND)raw;
    ShowWindow(window, SW_SHOWNOACTIVATE);
    BOOL positioned = SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    last_show_error = positioned ? ERROR_SUCCESS : GetLastError();
}

// Pair native show/hide: Tao's cached VISIBLE flag is not updated by fl_show.
void fl_hide(void *raw) {
    ShowWindow((HWND)raw, SW_HIDE);
}
bool fl_key_down(unsigned short code) { return (GetAsyncKeyState(code) & 0x8000) != 0; }

typedef struct {
    DWORD pid, integrity_rid, query_error, token_error, image_error;
    wchar_t executable[260];
} FlProcessInfo;

typedef struct {
    uintptr_t foreground_window;
    FlProcessInfo application, foreground;
    DWORD foreground_style, overlay_style;
    RECT foreground_rect, overlay_rect;
    int foreground_rect_valid, overlay_rect_valid, overlay_visible, foreground_above_overlay;
    DWORD overlay_cloaked, show_error;
    HRESULT cloak_result;
} FlWindowContext;

static void process_info(DWORD pid, FlProcessInfo *info) {
    info->pid = pid;
    HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if (!process) { info->query_error = GetLastError(); return; }
    wchar_t path[32768];
    DWORD length = sizeof(path) / sizeof(path[0]);
    if (QueryFullProcessImageNameW(process, 0, path, &length)) {
        const wchar_t *name = wcsrchr(path, L'\\');
        wcsncpy_s(info->executable, 260, name ? name + 1 : path, _TRUNCATE);
    } else info->image_error = GetLastError();

    HANDLE token;
    if (OpenProcessToken(process, TOKEN_QUERY, &token)) {
        // A mandatory label contains one SID; the union keeps its pointer aligned.
        union { TOKEN_MANDATORY_LABEL label; BYTE bytes[sizeof(TOKEN_MANDATORY_LABEL) + SECURITY_MAX_SID_SIZE]; } data;
        DWORD size;
        if (GetTokenInformation(token, TokenIntegrityLevel, &data, sizeof(data), &size)) {
            PSID sid = data.label.Label.Sid;
            info->integrity_rid = *GetSidSubAuthority(sid, *GetSidSubAuthorityCount(sid) - 1);
        } else info->token_error = GetLastError();
        CloseHandle(token);
    } else info->token_error = GetLastError();
    CloseHandle(process);
}

// Shortcut diagnostics only: executable basename and native window attributes.
// Never reads window titles, document paths, chat content, or unrelated keys.
void fl_window_context(void *raw, FlWindowContext *context) {
    HWND overlay = (HWND)raw;
    HWND foreground = GetForegroundWindow();
    DWORD pid = 0;
    GetWindowThreadProcessId(foreground, &pid);
    context->foreground_window = (uintptr_t)foreground;
    process_info(GetCurrentProcessId(), &context->application);
    process_info(pid, &context->foreground);
    context->foreground_style = (DWORD)GetWindowLongPtrW(foreground, GWL_EXSTYLE);
    context->foreground_rect_valid = GetWindowRect(foreground, &context->foreground_rect);
    context->overlay_style = (DWORD)GetWindowLongPtrW(overlay, GWL_EXSTYLE);
    context->overlay_rect_valid = GetWindowRect(overlay, &context->overlay_rect);
    context->overlay_visible = IsWindowVisible(overlay);
    context->cloak_result = DwmGetWindowAttribute(overlay, DWMWA_CLOAKED, &context->overlay_cloaked, sizeof(DWORD));
    context->show_error = last_show_error;
    context->foreground_above_overlay = -1;
    if (context->overlay_visible && foreground) {
        context->foreground_above_overlay = 0;
        HWND window = GetWindow(overlay, GW_HWNDPREV);
        unsigned steps = 0;
        // Z order may change during enumeration; bound diagnostic work on the UI thread.
        for (; window && steps < 256; window = GetWindow(window, GW_HWNDPREV), ++steps) {
            if (window == foreground) { context->foreground_above_overlay = 1; break; }
        }
        if (steps == 256) context->foreground_above_overlay = -1;
    }
}
static LRESULT CALLBACK session_proc(HWND w, UINT msg, WPARAM wp, LPARAM lp, UINT_PTR id, DWORD_PTR ref) {
    (void)id; (void)ref;
    if (hide_callback && ((msg == WM_WTSSESSION_CHANGE &&
            (wp == WTS_SESSION_LOCK || wp == WTS_SESSION_UNLOCK || wp == WTS_CONSOLE_DISCONNECT)) ||
            (msg == WM_POWERBROADCAST && (wp == PBT_APMSUSPEND || wp == PBT_APMRESUMEAUTOMATIC)))) {
        hide_callback();
    }
    if (displays_callback && (msg == WM_DISPLAYCHANGE || msg == WM_SETTINGCHANGE)) displays_callback();
    return DefSubclassProc(w, msg, wp, lp);
}
void fl_watch_session(void *raw, void (*hide)(void), void (*displays)(void)) {
    hide_callback = hide;
    displays_callback = displays;
    SetWindowSubclass((HWND)raw, session_proc, 1, 0);
    WTSRegisterSessionNotification((HWND)raw, NOTIFY_FOR_THIS_SESSION);
}

char *fl_choose_book(void *raw) {
    wchar_t path[32768] = {0};
    OPENFILENAMEW dialog = {0};
    dialog.lStructSize = sizeof(dialog);
    dialog.hwndOwner = (HWND)raw;
    dialog.lpstrFilter = L"Books (TXT, EPUB)\0*.txt;*.epub\0TXT files\0*.txt\0EPUB files\0*.epub\0\0";
    dialog.lpstrFile = path;
    dialog.nMaxFile = 32768;
    dialog.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if (!GetOpenFileNameW(&dialog)) return NULL;
    int count = WideCharToMultiByte(CP_UTF8, 0, path, -1, NULL, 0, NULL, NULL);
    char *result = (char *)malloc(count);
    if (result) WideCharToMultiByte(CP_UTF8, 0, path, -1, result, count, NULL, NULL);
    return result;
}
void fl_free_string(char *value) { free(value); }
