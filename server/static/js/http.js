function try_to_json(rawData) {
	var data = rawData;
	try {
		data = JSON.parse(rawData);
	} finally {
		return data;
	}
}

function qs(params) {
	if (params == null) {
		return '';
	}
	return Object
		.keys(params)
		.map(k => k + '=' + encodeURIComponent(params[k]))
		.join('&');
}

class http {
	static _prepareHeaders(xhr) {
		return xhr
			.getAllResponseHeaders()
			.split('\r\n')
			.filter(line => line.trim().length > 0)
			.map(gluedHeader => gluedHeader.split(':').map(part => part.trim()))
			.reduce((acc, pair) => {
				return Object.assign(acc, { [pair[0]]: pair[1] })
			}, {});
	}

	static request(method, addr, params, headers) {
		let xhr = new XMLHttpRequest();
		xhr.open(method, addr, true);
		for (let header of Object.keys(headers || {})) {
			let hvalue = headers[header];
			xhr.setRequestHeader(header, hvalue);
		}
		xhr.withCredentials = true;
		xhr.send(params);

		return new Promise((resolve, reject) => {
			xhr.onerror = reject;
			xhr.onload = _ => {
				if (xhr.status !== 200) {
					return reject(xhr.statusText);
				}
				let headers = http._prepareHeaders(xhr);

				return resolve({
					data: try_to_json(xhr.responseText),
					status: xhr.status,
					statusText: xhr.statusText,
					headers: headers
				});
			};
		});
	}

	static get(addr, params) {
		return http.request('GET', params ? addr + '?' + qs(params) : addr);
	}

	static post(addr, json) {
		if (typeof json == 'object') {
			json = JSON.stringify(json);
		}
		return http.request('POST', addr, json, {
			'Content-Type': 'application/json'
		});
	}

	static delete(addr, json) {
		if (typeof json == 'object') {
			json = JSON.stringify(json);
		}
		return http.request('delete', addr, json, {
			'Content-Type': 'application/json'
		});
	}

	static put(addr, json) {
		if (typeof json == 'object') {
			json = JSON.stringify(json);
		}
		return http.request('PUT', addr, json, {
			'Content-Type': 'application/json'
		});
	}
}