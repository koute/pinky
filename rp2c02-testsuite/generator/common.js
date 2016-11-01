"use strict";

var fs = require( "fs" );
var phantom2c02 = require( "./phantom2c02/phantom2c02" );
var _ = require( "./phantom2c02/lib/lodash" );
var sprintf = require( "./phantom2c02/lib/sprintf" ).sprintf;

function generate( name, callback ) {
    phantom2c02.create( function( ctx ) {
        var output = "";
        var automatically_output_code = true;

        var disable_automatic_code_output = function( callback ) {
            var previous = automatically_output_code;
            automatically_output_code = false;
            var result = callback();
            automatically_output_code = previous;

            return result;
        };

        var indent = function( callback ) {
            self.indentation += 4;
            callback();
            self.indentation -= 4;
        };

        var output_loop = function( times, callback, extra_args ) {
            extra_args = extra_args || {};

            self.println( "for %s in 0..%iu32 {", extra_args.variable || "i", times );
            indent( callback );

            if( extra_args.output_position === true ) {
                self.println( "} // %s -> %s", ctx.get_position().add( -1 ), ctx.get_position() );
            } else {
                self.println( "}" );
            }
        };

        var self = {
            indentation: 0,

            println: function() {
                var args = Array.prototype.slice.call( arguments );
                var line = sprintf.apply( null, args );
                self.repeat( self.indentation, function() {
                    output += " ";
                });
                output += line;
                output += "\n";
            },

            step_pixel: ctx.step_pixel,
            step_scanline: ctx.step_scanline,
            cpu_write_and_step: ctx.cpu_write_and_step,
            write_oam: ctx.write_oam,
            write_sprite_to_oam: ctx.write_sprite_to_oam,
            write_secondary_oam: ctx.write_secondary_oam,
            write_sprite_to_secondary_oam: ctx.write_sprite_to_secondary_oam,
            write_palette_ram: ctx.write_palette_ram,
            write_vram: ctx.write_vram,

            test: {
                next_vram_read: function() {
                    if( ctx.is_reading_from_vram() ) {
                        self.println( "ppu.expect_vram_read( 0x%04X, 0x%02X );", ctx.get_address_bus(), ctx.get_data_bus() );
                    } else {
                        self.println( "ppu.expect_no_vram_read();" );
                    }
                },

                current_address: function() {
                    self.println( "assert_eq( ppu.get_current_address(), 0x%04X );", ctx.get_current_address() );
                },

                temporary_address: function() {
                    self.println( "assert_eq( ppu.get_temporary_address(), 0x%04X );", ctx.get_temporary_address() );
                },

                cpu_read: function( address ) {
                    var obj = ctx.resolve_io_port( address );
                    ctx.queue_cpu_read( obj, function( value ) {
                        self.println( "assert_eq( ppu.read_ioreg( %i ), 0x%02X );", (obj.address - 0x2000) & (8 - 1), value );
                    });
                },

                secondary_oam_contents: function() {
                    _.each( ctx.dump_secondary_oam(), function( value, index ) {
                        self.println( "assert_eq( ppu.read_secondary_sprite_ram( %i ), 0x%02X );", index, value );
                    });
                }
            },

            fill_vram_with_test_data: function() {
                disable_automatic_code_output( function() {
                    // Fill out the first pattern table.
                    self.repeat( 0x1000, function( addr ) {
                        self.write_vram( addr, (addr + 0x80) & 0xFF );
                    });

                    // Fill out the first nametable.
                    self.repeat( 960, function( offset ) {
                        self.write_vram( 0x2000 + offset, (offset + 0x80) & 0xFF );
                    });

                    // Fill out the first attribute table.
                    self.repeat( 64, function( offset ) {
                        self.write_vram( 0x2000 + 960 + offset, 0x80 | offset );
                    });
                });

                output_loop( 0x1000, function() {
                    self.println( "ppu.write_vram( i as u16, (i + 0x80) as u8 );" );
                });

                output_loop( 960, function() {
                    self.println( "ppu.write_vram( (0x2000 + i) as u16, (i + 0x80) as u8 );" );
                });

                output_loop( 64, function() {
                    self.println( "ppu.write_vram( (0x2000 + 960 + i) as u16, (0x80 | i) as u8 );" );
                });
            },

            repeat: function( times, callback ) {
                if( callback === ctx.step_pixel ) {
                    disable_automatic_code_output( function() {
                        ctx.repeat( times, callback );
                        output_loop( times, _.partial( self.println, "ppu.step_pixel();" ), { output_position: true, variable: "_" } );
                    });
                } else if( callback === ctx.step_scanline ) {
                    disable_automatic_code_output( function() {
                        ctx.repeat( times, callback );
                        output_loop( times, _.partial( self.println, "ppu.step_scanline();" ), { output_position: true, variable: "_" } );
                    });
                } else {
                    ctx.repeat( times, callback );
                }
            },

            simulator: ctx
        };

        ctx.on_write_vram_called = function( address, value ) {
            if( !automatically_output_code ) return;
            self.println( "ppu.write_vram( 0x%04X, 0x%02X );", address, value );
        };

        ctx.on_write_palette_ram_called = function( offset, value ) {
            if( !automatically_output_code ) return;
            self.println( "ppu.write_palette_ram( %i, 0x%02X );", offset, value );
        };

        ctx.on_write_oam_called = function( offset, value ) {
            if( !automatically_output_code ) return;
            self.println( "ppu.write_sprite_ram( %i, 0x%02X );", offset, value );
        };

        ctx.on_write_secondary_oam_called = function( offset, value ) {
            if( !automatically_output_code ) return;
            self.println( "ppu.write_secondary_sprite_ram( %i, 0x%02X );", offset, value );
        };

        ctx.on_cpu_write_called = function( address, value ) {
            if( !automatically_output_code ) return;
            self.println( "ppu.write_ioreg( %i, 0x%02X );", (address - 0x2000) & (8 - 1), value );
        };

        ctx.on_step_pixel_called = function() {
            if( !automatically_output_code ) return;
            self.println( "ppu.step_pixel(); // %s -> %s", ctx.get_position().add( -1 ), ctx.get_position() );
        };

        ctx.on_step_scanline_called = function() {
            if( !automatically_output_code ) return;
            self.println( "ppu.step_scanline();" );
        };

        console.log( "Generating '" + name + "'..." );

        self.println( "// This file was AUTOGENERATED; do not edit!" );
        self.println( "" );
        self.println( "#[allow(unused_imports)]" );
        self.println( "use super::super::assert_eq;" );
        self.println( "use TestPPU;" );
        self.println( "" );
        self.println( "#[inline(never)]" );
        self.println( "pub fn test_" + name + "( ppu: &mut TestPPU ) {" );
        self.indentation += 4;
        self.println( "assert_eq!( ppu.scanline(), %s );", ctx.get_scanline() );
        self.println( "assert_eq!( ppu.dot(), %s );", ctx.get_dot() );
        callback( self );
        self.println( "assert_eq!( ppu.scanline(), %s );", ctx.get_scanline() );
        self.println( "assert_eq!( ppu.dot(), %s );", ctx.get_dot() );
        self.indentation -= 4;
        self.println( "}" );

        fs.write( module.dirname + "/../src/tests/test_" + name + ".rs", output );
        ctx.exit();
    });
}

module.exports.generate = generate;
module.exports.ppumask = phantom2c02.ppumask;
module.exports.ppuctrl = phantom2c02.ppuctrl;
module.exports.sprite = phantom2c02.sprite;
