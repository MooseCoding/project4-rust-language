const std = @import("std");

pub fn main() !void {
    var args = std.process.args();
    _ = args.next(); // skip program name

    const mode = args.next() orelse {
        return usage();
    };

    const ip = args.next() orelse {
        return usage();
    };

    const port_str = args.next() orelse {
        return usage();
    };

    const port = try std.fmt.parseInt(u16, port_str, 10);

    if (std.mem.eql(u8, mode, "server")) {
        try runServer(ip, port);
    } else if (std.mem.eql(u8, mode, "client")) {
        try runClient(ip, port);
    } else {
        return usage();
    }
}

fn usage() !void {
    std.debug.print("Usage: zig run main.zig -- [server|client] [ip] [port]\n", .{});
    return error.InvalidArguments;
}

fn runServer(ip: []const u8, port: u16) !void {
    const address = try std.net.Address.parseIp(ip, port);
    var server = try std.net.Stream.server.init(.{});
    defer server.deinit();

    try server.listen(address);
    std.debug.print("Server listening on {s}:{d}\n", .{ ip, port });

    while (true) {
        var conn = try server.accept();
        defer conn.stream.close();

        std.debug.print("Client connected!\n", .{});

        var buf: [1024]u8 = undefined;
        const len = try conn.stream.reader().read(&buf);
        const received = buf[0..len];
        std.debug.print("Server received: {s}\n", .{received});

        try conn.stream.writer().writeAll(received);
        std.debug.print("Server echoed message.\n", .{});
    }
}

fn runClient(ip: []const u8, port: u16) !void {
    const address = try std.net.Address.parseIp(ip, port);
    var stream = try std.net.Stream.connect(address);
    defer stream.close();

    const msg = "Hello from Zig client!";
    try stream.writer().writeAll(msg);
    std.debug.print("Client sent: {s}\n", .{msg});

    var buf: [1024]u8 = undefined;
    const len = try stream.reader().read(&buf);
    const response = buf[0..len];
    std.debug.print("Client received: {s}\n", .{response});
}