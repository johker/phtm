
class LinearSamplingEncoder {

    middle_out_range(value) {
	let start = Math.round(value);
	let rounded_down = (start > value);

    }
    
    multiples_within_radius(center, radius, multiples_of) {
        let lower_bound = center - radius;
        let upper_bound = center + radius;
	let steps = center / multiples_of;
	return [];
	


    }
}

module.exports = LinearSamplingEncoder;
