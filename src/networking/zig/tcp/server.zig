const std = @import("std"); 

pub fn main(iP: []u8, port: u16) !void {
    const addr = try std.net.Address.parseIp(iP, port);

    var server = try std.net.StreamServer.init(.{});
    defer server.deinit(); 

    try server.listen(addr);
    std.debug.print("Server listening on {s}:{d}", .{iP, port}); 

    while (true) {
        var conn = try server.accept(); 
        defer conn.stream.close(); 

        std.debug.print("Client connection!\n", .{});

        var buff: [1024]u8 = undefined;
        const len = try conn.stream.reader().read(&buff);
        const received = buff[0..len];

        std.debug.print("Received: {s}\n", .{received});

        try conn.stream.writer().writeAll(received);
        std.debug.print("Server echoed message\n", .{});    
    }

    
}  