#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commctrl.h>
#include <wtsapi32.h>
#include <stdbool.h>

static void (*hide_callback)(void);
void fl_show(void *raw) {
    HWND window = (HWND)raw;
    ShowWindow(window, SW_SHOWNOACTIVATE);
    SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
}
bool fl_key_down(unsigned short code) { return (GetAsyncKeyState(code) & 0x8000) != 0; }
static LRESULT CALLBACK session_proc(HWND w, UINT msg, WPARAM wp, LPARAM lp, UINT_PTR id, DWORD_PTR ref) {
    (void)id; (void)ref;
    if (hide_callback && ((msg == WM_WTSSESSION_CHANGE &&
            (wp == WTS_SESSION_LOCK || wp == WTS_SESSION_UNLOCK || wp == WTS_CONSOLE_DISCONNECT)) ||
            (msg == WM_POWERBROADCAST && (wp == PBT_APMSUSPEND || wp == PBT_APMRESUMEAUTOMATIC)))) {
        hide_callback();
    }
    return DefSubclassProc(w, msg, wp, lp);
}
void fl_watch_session(void *raw, void (*hide)(void)) {
    hide_callback = hide;
    SetWindowSubclass((HWND)raw, session_proc, 1, 0);
    WTSRegisterSessionNotification((HWND)raw, NOTIFY_FOR_THIS_SESSION);
}
