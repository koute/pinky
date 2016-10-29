"use strict";

var _ = require( "./phantom2c02/lib/lodash" );
var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "vram_access_sprite_rendering_out_of_range", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_sprites( true )
        .set_show_sprites_in_leftmost_8_pixels( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.step_scanline();

    ctx.repeat( 7, ctx.step_scanline );
    ctx.repeat( 2, function() {
        ctx.repeat( 257, ctx.step_pixel ); // Skip sprite evaluation.
        // Technically some of these will never appear on one of the tested scanlines,
        // but it's a good idea to test it anyway to figure out how the PPU calculates
        // the sprite pattern addresses.
        ctx.write_sprite_to_secondary_oam( 0, sprite({ x: 0, y: 0, tile: 0x77 }) );
        ctx.write_sprite_to_secondary_oam( 1, sprite({ x: 0, y: 1, tile: 0x88 }) );
        ctx.write_sprite_to_secondary_oam( 2, sprite({ x: 0, y: 7, tile: 0x99 }) );
        ctx.write_sprite_to_secondary_oam( 3, sprite({ x: 0, y: 8, tile: 0xAA }) );
        ctx.write_sprite_to_secondary_oam( 4, sprite({ x: 0, y: 9, tile: 0xBB }) );
        ctx.write_sprite_to_secondary_oam( 5, sprite({ x: 0, y: 15, tile: 0xCC }) );
        ctx.write_sprite_to_secondary_oam( 6, sprite({ x: 0, y: 16, tile: 0xDD }) );
        ctx.write_sprite_to_secondary_oam( 7, sprite({ x: 0, y: 255, tile: 0xEE }) );

        /*
            | SCANLINE |  Y  | OFFSET |
            |     7    |   0 |   7    |
            |     7    |   1 |   6    |
            |     7    |   2 |   5    |
            |     7    |   3 |   4    |
            |     7    |   4 |   3    |
            |     7    |   5 |   2    |
            |     7    |   6 |   1    |
            |     7    |   7 |   0    |
            |     7    |   8 |   7    |
            |     7    |   9 |   6    |
            |     7    |  10 |   5    |
            |     7    |  11 |   4    |
            |     7    |  12 |   3    |
            |     7    |  15 |   0    |
            |     7    |  16 |   7    |
            |     7    | 255 |   0    |
            |     8    |   0 |   0    |
            |     8    |   1 |   7    |
            |     8    |   2 |   6    |
            |     8    |   3 |   5    |
            |     8    |   4 |   4    |
            |     8    |   5 |   3    |
            |     8    |   6 |   2    |
            |     8    |   7 |   1    |
            |     8    |   8 |   0    |
            |     8    |   9 |   7    |
            |     8    |  10 |   6    |
            |     8    |  11 |   5    |
            |     8    |  12 |   4    |
            |     8    |  15 |   1    |
            |     8    |  16 |   0    |
            |     8    | 255 |   1    |
        */

        ctx.repeat( 64, function() {
            ctx.test.next_vram_read();
            ctx.step_pixel();
        });
        ctx.step_scanline();
    });

});
