const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Hello, {s}!\n", .{"Zig"});
}

fn add_numbers(a: i32, b: i32) i32 {
    // Zig function logic
    return a + b;
}
