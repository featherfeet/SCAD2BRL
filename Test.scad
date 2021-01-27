$fn = 100;

difference() {
    cylinder(d = 10, h = 10);
    translate([0, 0, -5])
        cylinder(d = 5, h = 20);
}
