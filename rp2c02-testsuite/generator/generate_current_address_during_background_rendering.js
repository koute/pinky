"use strict";

var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "current_address_during_background_rendering", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_background( true )
        .set_show_background_in_leftmost_8_pixels( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.step_scanline();

    var test = function() {
        ctx.test.current_address();
        ctx.step_pixel();
    };

    ctx.repeat( 2 * 341, test );
    ctx.repeat( 5, ctx.step_scanline );
    ctx.repeat( 2 * 341, test );
});
