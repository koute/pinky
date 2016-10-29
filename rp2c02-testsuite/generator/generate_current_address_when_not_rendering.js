"use strict";

var _ = require( "./phantom2c02/lib/lodash" );
var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "current_address_when_not_rendering", function( ctx ) {
    ctx.fill_vram_with_test_data();
    ctx.step_scanline();

    var test = function() {
        ctx.test.current_address();
        ctx.step_pixel();
    };

    ctx.repeat( 2 * 341, test );
    ctx.repeat( 5, ctx.step_scanline );
    ctx.repeat( 2 * 341, test );
});
