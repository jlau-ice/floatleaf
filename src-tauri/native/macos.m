#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <WebKit/WebKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#import <Carbon/Carbon.h>

// No key/frontmost activation, including when FloatLeaf is not the active app.
void fl_show(void *raw) {
    NSWindow *window = (__bridge NSWindow *)raw;
    [window orderFrontRegardless];
}

bool fl_key_down(unsigned short code) {
    return CGEventSourceKeyState(kCGEventSourceStateCombinedSessionState, code);
}

void fl_watch_session(void *raw, void (*hide)(void), void (*displays)(void)) {
    (void)raw;
    static NSMutableArray *tokens;
    tokens = [NSMutableArray new];
    [tokens addObject:[[NSNotificationCenter defaultCenter] addObserverForName:NSApplicationDidChangeScreenParametersNotification object:nil queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *note) { (void)note; displays(); }]];
    NSNotificationCenter *workspace = [[NSWorkspace sharedWorkspace] notificationCenter];
    for (NSString *name in @[NSWorkspaceWillSleepNotification, NSWorkspaceDidWakeNotification,
                            NSWorkspaceScreensDidSleepNotification, NSWorkspaceSessionDidResignActiveNotification]) {
        [tokens addObject:[workspace addObserverForName:name object:nil queue:[NSOperationQueue mainQueue]
                                            usingBlock:^(NSNotification *note) { (void)note; hide(); }]];
    }
    for (NSString *name in @[@"com.apple.screenIsLocked", @"com.apple.screenIsUnlocked"]) {
        [tokens addObject:[[NSDistributedNotificationCenter defaultCenter]
            addObserverForName:name object:nil queue:[NSOperationQueue mainQueue]
            usingBlock:^(NSNotification *note) { (void)note; hide(); }]];
    }
}

// Used by the opt-in development probe; does not request or change permissions.
bool fl_automation_allowed(void) { return AXIsProcessTrusted(); }
bool fl_window_clickthrough(void *raw) { return [(__bridge NSWindow *)raw ignoresMouseEvents]; }
bool fl_window_key(void *raw) { return [(__bridge NSWindow *)raw isKeyWindow]; }
bool fl_window_opaque(void *raw) { return [(__bridge NSWindow *)raw isOpaque]; }
long fl_window_level(void *raw) { return [(__bridge NSWindow *)raw level]; }
int fl_frontmost_pid(void) { return [[[NSWorkspace sharedWorkspace] frontmostApplication] processIdentifier]; }

char *fl_choose_book(void *raw) {
    NSOpenPanel *panel = [NSOpenPanel openPanel];
    panel.canChooseDirectories = NO;
    panel.allowsMultipleSelection = NO;
    panel.allowedContentTypes = @[[UTType typeWithFilenameExtension:@"txt"], [UTType typeWithFilenameExtension:@"epub"]];
    panel.title = @"导入 TXT / EPUB 小说";
    panel.message = @"仅复制文件到浮页，原文件不会被修改。";
    [(__bridge NSWindow *)raw makeKeyAndOrderFront:nil];
    if ([panel runModal] != NSModalResponseOK) return NULL;
    return strdup(panel.URL.path.UTF8String);
}
void fl_free_string(char *value) { free(value); }

static WKWebView *findWebView(NSView *view) {
    if ([view isKindOfClass:[WKWebView class]]) return (WKWebView *)view;
    for (NSView *child in view.subviews) {
        WKWebView *result = findWebView(child);
        if (result) return result;
    }
    return nil;
}
// Capture only our own webview, without reading the desktop or other apps.
void fl_snapshot(void *raw, const char *path) {
    NSString *destination = [NSString stringWithUTF8String:path];
    WKWebView *webview = findWebView([(__bridge NSWindow *)raw contentView]);
    [webview takeSnapshotWithConfiguration:nil completionHandler:^(NSImage *image, NSError *error) {
        if (error || !image) return;
        NSBitmapImageRep *rep = [NSBitmapImageRep imageRepWithData:image.TIFFRepresentation];
        [[rep representationUsingType:NSBitmapImageFileTypePNG properties:@{}] writeToFile:destination atomically:YES];
    }];
}

// Carbon's shared registration can succeed even when an exclusive owner exists.
// Check that case explicitly before the plugin installs its shared registration.
int fl_check_hotkey(unsigned short code, unsigned int modifiers) {
    EventHotKeyRef key;
    EventHotKeyID identity = { .signature = 'FLCK', .id = 1 };
    OSStatus result = RegisterEventHotKey(code, modifiers, identity, GetApplicationEventTarget(), kEventHotKeyExclusive, &key);
    if (result == noErr) UnregisterEventHotKey(key);
    return result;
}

// Diagnostic process attribution for our own WebViews (not desktop capture).
@interface WKWebView (FloatLeafDiagnostics)
- (pid_t)_webProcessIdentifier;
@end
int fl_web_pid(void *raw) {
    WKWebView *view = findWebView([(__bridge NSWindow *)raw contentView]);
    if (![view respondsToSelector:@selector(_webProcessIdentifier)]) return 0;
    return [view _webProcessIdentifier];
}
