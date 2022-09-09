
const LinearSamplingEncoder = require('../src/encoder.js');

describe('multiples_within_radius', () => {
	it('creates sequence within bounds', () => {
		let encoder = new LinearSamplingEncoder();
		expect(encoder.multiples_within_radius(100,10,2)).toEqual([]);
	});
});
