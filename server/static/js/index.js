window.addEventListener('DOMContentLoaded', run);

class Templator {
	constructor(text, fields) {
		this.text = text;
		this.fields = fields;
	}

	build(data) {
		let completion = this.text;
		for (let field of this.fields) {
			let subst = data.hasOwnProperty(field) ? data[field] : "";
			completion = completion
				.split('((' + field + '))')
				.join(subst);
		}
		let elem = document.createElement('div');
		elem.innerHTML = completion;
		return elem;
	}

	static fields(t) {
		let fields = t.match(new RegExp(/\(\([a-zA-Z0-9-]+\)\)/g)) || [];
		return fields.map(field => field.slice(2).slice(0, -2))
	}

	static byId(templateId) {
		let text = document.getElementById(templateId).innerHTML;
		let fields = Templator.fields(text);

		return new Templator(text, fields)
	}
}
function run() {
	loadFeeds()
		.then(buildItems)
		.then(showFeeds)
		.catch(showError);
}

function loadFeeds() {
	return http.get('/feed').then(r => r.data);
}

function buildItems(feeds) {
	console.log(feeds);
	let fragment = document.createDocumentFragment();
	let template = Templator.byId('feed-template');
	feeds
		.map(feed => template.build(feed))
		.forEach(elem => fragment.appendChild(elem));
	return fragment;
}

function showFeeds(elem) {
	document.querySelector('#feed-list').appendChild(elem);
}

function showError(e) {
	console.error(e);
}