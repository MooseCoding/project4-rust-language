const std = @import("std");

pub fn main(iP: []u8, port: u16) !void {
    const addr = try std.net.Address.parseIp(iP, port);
    var stream = try std.net.Stream.connect(addr);
    defer stream.close();

    const msg = "Hello from Client!";
    try stream.writer().writeAll(msg);

    std.debug.print("Client sent: {s}\n", .{msg});

    var buff: [1024]u8 = undefined;
    const len = try stream.reader().read(&buff);
    const response = buff[0..len];

    std.debug.print("Received {s}\n", .{response});
}