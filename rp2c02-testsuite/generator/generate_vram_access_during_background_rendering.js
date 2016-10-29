"use strict";

var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "vram_access_during_background_rendering", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_background( true )
        .set_show_background_in_leftmost_8_pixels( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.step_scanline();

    ctx.repeat( 257, function() {
        ctx.test.next_vram_read();
        ctx.step_pixel();
    });
    ctx.repeat( 64, ctx.step_pixel );
    ctx.repeat( 20, function() {
        ctx.test.next_vram_read();
        ctx.step_pixel();
    });
});
