$fn = 100;

difference() {
    cylinder(d = 10, h = 10);
    translate([0, 0, -5])
        cylinder(d = 8, h = 20);
    for (height = [1 : 9]) {
        for (angle = [0 : 360/20 : 360]) {
            translate([0, 0, height])
                rotate([0, 90, angle])
                    cylinder(d = 0.5, h = 10);
        }
    }
}
