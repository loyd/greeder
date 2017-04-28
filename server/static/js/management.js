window.addEventListener('DOMContentLoaded', rebuildLists);

function rebuildLists() {

	load()
		.then(buildSubs)
		.then(showSubs)
		.catch(showError);
}

function load() {
	return http.get('/management/feed_n_subs').then(r => r.data);
}

function unsubHandler(e) {
	let feed_id = e.target.getAttribute('data-unsub');
	http.post('/management/unsub', { feed_id })
		.then(rebuildLists)
}

function subHandler(e) {
	let feed_id = e.target.getAttribute('data-sub');
	http.post('/management/sub', { feed_id })
		.then(rebuildLists);
}

function buildSubs(feeds) {
	let subs = feeds.subs;
	let other = feeds.other_feeds;

	let subFragment = document.createDocumentFragment();
	let feedFragment = document.createDocumentFragment();
	let subTemplate = Templator.byId('sub-template');
	let feedTemplate = Templator.byId('feed-template');

	subs
		.map(feed => subTemplate.build(feed))
		.map(elem => {
			elem.querySelector('.unsub-btn').addEventListener('click', unsubHandler);
			return elem;
		})
		.forEach(elem => subFragment.appendChild(elem));

	other
		.map(feed => feedTemplate.build(feed))
		.map(elem => {
			elem.querySelector('.sub-btn').addEventListener('click', subHandler);
			return elem;
		})
		.forEach(elem => feedFragment.appendChild(elem));

	return [subFragment, feedFragment];
}

function showSubs(fragments) {
	let [subList, feedList] = fragments;
	let subListElem = document.querySelector('#subscription-list');
	let feedListElem = document.querySelector('#other-list');
	subListElem.innerHTML = '';
	feedListElem.innerHTML = '';

	subListElem.appendChild(subList);
	feedListElem.appendChild(feedList);
}

function showError(e) {
	console.error(e);
}