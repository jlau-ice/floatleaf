#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <WebKit/WebKit.h>

// No key/frontmost activation, including when FloatLeaf is not the active app.
void fl_show(void *raw) {
    NSWindow *window = (__bridge NSWindow *)raw;
    [window orderFrontRegardless];
}

bool fl_key_down(unsigned short code) {
    return CGEventSourceKeyState(kCGEventSourceStateCombinedSessionState, code);
}

void fl_watch_session(void *raw, void (*hide)(void)) {
    (void)raw;
    static NSMutableArray *tokens;
    tokens = [NSMutableArray new];
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
