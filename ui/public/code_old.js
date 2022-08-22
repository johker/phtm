var socket = io();
var ploff = 8;
var n = 4096;
var cmax = 64;

function sendData(key, value) {
	socket.emit('data', {key,value});
}

function sendConfig(key, value) {
	socket.emit('config', {key,value});
}

socket.on('sdr', (msg) => {
	let arrayView = new Uint8Array(msg);
	for(idx = 0; idx < n; idx++) {
		gridData[idx].active = is_active(arrayView, idx);
	}
	updateGrid();
});


function is_active(msg, idx) {
	let byteIdx = (idx >> 3) + ploff;
	let bitIdx = idx % 8;
	return (msg[byteIdx] >> bitIdx) % 2 != 0;
}


function initGridData() {
	var data = new Array();
	var xpos = 1; //starting xpos and ypos at 1 so the stroke will show when we make the grid below
	var ypos = 1;
	var width = 5;
	var height = 5;
	var offset = 5;
	var id = 1;
	var rmax = Math.round(n / cmax);
	// iterate for rows 
	for (var cidx = 0; cidx < cmax; cidx++) {

	        // iterate for cells/columns inside rows
		for (var ridx = 0; ridx < rmax; ridx++) {
        		data.push({
			id: id,
			x: xpos,
			y: ypos,
			width: width,
			height: height,
			active: false	
		})
		xpos += width + offset;
		id += 1;
	}
	// reset the x position after a row is complete
	xpos = 1;
	ypos += height + offset;
	}
	console.log("ID = " + id);
    return data;
}

function updateGrid() {
	 svg.selectAll("rect")
		.data(gridData, d => d.id)
		.join(
			enter => enter.append('rect')
			.attr("class","square")
			.attr("x", function(d) { return d.x; })
			.attr("y", function(d) { return d.y; })
			.attr("width", function(d) { return d.width; })
			.attr("height", function(d) { return d.height; })
			.attr("fill", function(d) { return d.active ? "#00aaff" : "#999999";}),
			update => update
			.attr("fill", function(d) { return d.active ? "#00aaff" : "#999999";}),
		);
	
}

var gridData = initGridData();	
// I like to log the data to the console for quick debugging

var svg = d3.select("#grid")
	.append("svg")
	.attr("width","640px")
	.attr("height","640px");
	
updateGrid();


// convenience function to update everything (run after UI input)
function updateAll() {
	console.log('UpdateAll');
    // updateForces();
    // updateDisplay();
}
