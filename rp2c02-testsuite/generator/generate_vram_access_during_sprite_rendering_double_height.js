"use strict";

var _ = require( "./phantom2c02/lib/lodash" );
var common = require( "./common" );
var ppumask = common.ppumask;
var ppuctrl = common.ppuctrl;
var sprite = common.sprite;

common.generate( "vram_access_during_sprite_rendering_double_height", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_sprites( true )
        .set_show_sprites_in_leftmost_8_pixels( true );

    var ppuctrl_value = ppuctrl()
        .set_double_height_sprite_mode( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.cpu_write_and_step( ppuctrl_value );
    ctx.step_scanline();

    var test = function() {
        ctx.repeat( 257, ctx.step_pixel ); // Skip sprite evaluation.
        ctx.repeat( 32, _.partial( ctx.write_secondary_oam, _, 0xFF ) );

        ctx.write_sprite_to_secondary_oam( 0, sprite({ x: 0, y: 0, tile: 0xAA }) );
        ctx.write_sprite_to_secondary_oam( 1, sprite({ x: 8, y: 0, tile: 0xBA, flip_h: true }) );
        ctx.write_sprite_to_secondary_oam( 2, sprite({ x: 16, y: 0, tile: 0xCA, flip_v: true }) );
        ctx.write_sprite_to_secondary_oam( 3, sprite({ x: 24, y: 0, tile: 0xDA, flip_h: true, flip_v: true }) );

        ctx.repeat( 64, function() {
            ctx.test.next_vram_read();
            ctx.step_pixel();
        });
        ctx.step_scanline();
    };

    ctx.repeat( 11, test );
});
