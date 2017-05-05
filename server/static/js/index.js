window.addEventListener('DOMContentLoaded', run);

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