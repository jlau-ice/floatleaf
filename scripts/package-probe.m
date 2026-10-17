// Read-only check of the installed app's own visible window metadata; no pixels or other app content.
// clang -fobjc-arc -framework AppKit -framework ApplicationServices scripts/package-probe.m -o .local/package-probe
#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
int main(int argc, const char **argv) { @autoreleasepool {
    if (argc != 2) return 2;
    pid_t pid = atoi(argv[1]);
    NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:pid];
    NSMutableArray *ownWindows = [NSMutableArray new];
    NSArray *windows = CFBridgingRelease(CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, kCGNullWindowID));
    for (NSDictionary *window in windows) {
        if ([window[(id)kCGWindowOwnerPID] intValue] == pid) {
            [ownWindows addObject:@{ @"bounds": window[(id)kCGWindowBounds], @"layer": window[(id)kCGWindowLayer], @"alpha": window[(id)kCGWindowAlpha] }];
        }
    }
    NSDictionary *result = @{ @"pid": @(pid), @"running": @((app != nil) && !app.terminated), @"identifier": app.bundleIdentifier ?: @"", @"ownVisibleWindows": ownWindows };
    NSData *json = [NSJSONSerialization dataWithJSONObject:result options:NSJSONWritingPrettyPrinted error:nil];
    puts([[NSString alloc] initWithData:json encoding:NSUTF8StringEncoding].UTF8String);
    return app && !app.terminated && ownWindows.count > 0 ? 0 : 1;
} }
