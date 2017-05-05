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

function addUrlHandler(e) {
	let urlInput = document.querySelector('#url-add');
	let statusLabel = document.querySelector('#addition-status');
	let url = urlInput.value;
	if (url.trim().length === 0) return;

	http.post('/feed/add', { url }).then(_ => {
		urlInput.value = '';
		statusLabel.innerHTML = 'Лента скоро будет добавлена';
		setTimeout(_ => {
			statusLabel.innerHTML = '&nbsp';
		}, 4000);
	}).catch(err => {
		statusLabel.innerHTML = '<div style="color: red;">Неверный URL</div>';
		statusLabel.classList.add('list-fade-in');
		setTimeout(_ => {
			statusLabel.innerHTML = '&nbsp';
		}, 2000);
	})
}

function run() {
	[...document.querySelectorAll('.unsub-btn')].forEach(btn => {
		btn.addEventListener('click', createUnsubHandler(btn));
	});
	document.querySelector('#url-add-button').addEventListener('click', addUrlHandler);
}