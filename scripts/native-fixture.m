// A separate foreground app for FloatLeaf focus/click-through checks.
// Build: clang -fobjc-arc -framework AppKit -framework Carbon scripts/native-fixture.m -o .local/native-fixture
#import <AppKit/AppKit.h>
#import <Carbon/Carbon.h>
@interface ProbeView : NSTextView
@end
@implementation ProbeView
- (void)mouseDown:(NSEvent *)event {
    fprintf(stderr, "FIXTURE_MOUSE_DOWN %.3f\n", event.timestamp); fflush(stderr);
    [super mouseDown:event];
}
- (void)keyDown:(NSEvent *)event {
    fprintf(stderr, "FIXTURE_KEY_DOWN %.3f code=%u\n", event.timestamp, event.keyCode); fflush(stderr);
    [super keyDown:event];
}
@end
int main(int argc, const char **argv) { @autoreleasepool {
    if (argc == 2 && strcmp(argv[1], "--hotkey") != 0) {
        NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:atoi(argv[1])];
        [app activateWithOptions:NSApplicationActivateIgnoringOtherApps];
        return 0;
    }
    [NSApplication sharedApplication];
    if (argc == 2) {
        EventHotKeyRef key; EventHotKeyID identity = { .signature = 'FLFT', .id = 1 };
        OSStatus result = RegisterEventHotKey(111, controlKey | optionKey, identity, GetApplicationEventTarget(), kEventHotKeyExclusive, &key);
        fprintf(stderr, "FIXTURE_HOTKEY Control+Alt+F12 status=%d\n", (int)result); fflush(stderr);
    }
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
    NSWindow *window = [[NSWindow alloc] initWithContentRect:NSMakeRect(150, 350, 760, 340)
        styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable backing:NSBackingStoreBuffered defer:NO];
    window.title = @"FloatLeaf 原生验收输入窗口（独立进程）";
    ProbeView *view = [[ProbeView alloc] initWithFrame:window.contentView.bounds];
    view.font = [NSFont systemFontOfSize:22];
    view.string = @"这是独立进程的输入窗口。\n显示浮页时，输入焦点应保持在这里。\n鼠标穿透与真实按键需手工验收。";
    window.contentView = view;
    [window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    fprintf(stderr, "FIXTURE_PID %d\n", [[NSProcessInfo processInfo] processIdentifier]); fflush(stderr);
    [NSApp run];
} }
