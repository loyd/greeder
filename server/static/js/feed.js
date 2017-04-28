window.addEventListener('DOMContentLoaded', run);

function run() {
	loadEntries()
		.then(buildItems)
		.then(showEntries)
		.catch(showError);
}

function loadFeeds() {
	let entryId = document.querySelector('.content').getAttribute('data-feed-id');
	return http.get('/feed/entries/'+entryId).then(r => r.data);
}

function buildItems(feeds) {
	console.log(feeds);
	let fragment = document.createDocumentFragment();
	let template = Templator.byId('entry-template');
	feeds
		.map(feed => template.build(feed))
		.forEach(elem => fragment.appendChild(elem));
	return fragment;
}

function showEntries(elem) {
	document.querySelector('#feed-list').appendChild(elem);
}

function showError(e) {
	console.error(e);
}