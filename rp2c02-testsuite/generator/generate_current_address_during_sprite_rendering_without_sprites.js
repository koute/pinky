"use strict";

var _ = require( "./phantom2c02/lib/lodash" );
var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "current_address_during_sprite_rendering_without_sprites", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_sprites( true )
        .set_show_sprites_in_leftmost_8_pixels( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.step_scanline();

    var test = function() {
        ctx.repeat( 257, ctx.step_pixel ); // Skip sprite evaluation.
        ctx.test.secondary_oam_contents(); // This should contain [0x00, 0xFF, 0xFF, 0xFF, ...].

        ctx.repeat( 64, function() {
            ctx.test.current_address();
            ctx.step_pixel();
        });
        ctx.step_scanline();
    };

    ctx.repeat( 24, test );
});
