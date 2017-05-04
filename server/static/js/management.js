document.addEventListener('DOMContentLoaded', run);

function createUnsubHandler(btn) {
	let feedEntry = btn.parentNode.parentNode.parentNode;
	let key = btn.getAttribute('data-key');

	return e => {
		http.post('/management/unsub', { key }).then(data => {
			feedEntry.classList.add('hide-transition');
		});
	};
}

function run() {
	[...document.querySelectorAll('.unsub-btn')].forEach(btn => {
		btn.addEventListener('click', createUnsubHandler(btn));
	});
}