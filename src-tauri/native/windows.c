#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>
#include <wtsapi32.h>
#include <stdbool.h>
#include <commdlg.h>
#include <stdlib.h>

static void (*hide_callback)(void);
static void (*displays_callback)(void);
void fl_show(void *raw) {
    HWND window = (HWND)raw;
    ShowWindow(window, SW_SHOWNOACTIVATE);
    SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
}

// Pair native show/hide: Tao's cached VISIBLE flag is not updated by fl_show.
void fl_hide(void *raw) {
    ShowWindow((HWND)raw, SW_HIDE);
}
bool fl_key_down(unsigned short code) { return (GetAsyncKeyState(code) & 0x8000) != 0; }
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
