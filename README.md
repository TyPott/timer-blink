# Timer Blink Example

Expands on the traditional "blinky" example by using a timer peripheral to control when to toggle the LED. This repo uses the `sparkfun_thing_plus_rp2040` crate, but really only uses the LED pin alias instead of "gpio25" and the frequency of the external crystal oscillator on the board. It should be trivial to port the example to any platform using an RP2040.
