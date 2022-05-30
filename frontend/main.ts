window.onload = init;

// get an element by id or throw an exception
function getElement(id: string): HTMLElement {
	let elem = document.getElementById(id);
	if(!elem) {
		throw new Error('missing element #' + id);
	}
	return elem;
}

function init() {
	console.log('init');
	(<HTMLElement>getElement('theme')).onchange = function(e) {
		let t = (<HTMLSelectElement>getElement('theme')).value;
		let hc = getElement('h').classList;
		if(t == 'night' || t == 'light') {
			document.cookie = `theme=${t};path=/;SameSite=Strict`;
		} else {
			document.cookie = `theme=;path=/;SameSite=Strict;expires=Thu, 01 Jan 1970 00:00:01 GMT`;
		}
		if(!t.length) {
			let h = document.location.host;
			if(h.startsWith('light.') || h.startsWith('day.')) {
				t = 'light';
			} else if(h.startsWith('night.') || h.startsWith('dark.')) {
				t = 'night';
			}
		}
		if(t == 'night' || t == 'light') { hc.add(t); }
		if(t != 'light') { hc.remove('light'); }
		if(t != 'night') { hc.remove('night'); }
	};
}

