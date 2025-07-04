import './style.css';

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <h1>Live Financial Stream (SSE)</h1>
    <div>
      <input type="text" id="ticker-input" placeholder="Enter Ticker (e.g., AAPL)" value="AAPL"/>
      <button id="start-stream-button">Start Stream</button>
    </div>
    <div id="data-container"></div>
  </div>
`;

const tickerInput = document.querySelector<HTMLInputElement>('#ticker-input');
const startStreamButton = document.querySelector<HTMLButtonElement>('#start-stream-button');
const dataContainer = document.querySelector<HTMLDivElement>('#data-container');

let eventSource: EventSource | null = null;

function startStream(ticker: string) {
	if (eventSource) {
		eventSource.close();
		eventSource = null;
	}
	if (dataContainer) {
		dataContainer.innerHTML = ''; // Clear previous data
	}

	const url = `http://127.0.0.1:3000?ticker=${ticker}`;
	eventSource = new EventSource(url);

	eventSource.onmessage = (event) => {
		const quote = JSON.parse(event.data);
		const quoteElement = document.createElement('div');
		quoteElement.innerHTML = `<strong>${quote.symbol}</strong>: $${quote.price.toFixed(2)} @ ${new Date(quote.timestamp * 1000).toLocaleTimeString()}`;
		dataContainer?.prepend(quoteElement); // Add new data at the top
	};

	eventSource.onerror = (err) => {
		console.error("EventSource failed:", err);
		eventSource?.close();
	};
}

startStreamButton?.addEventListener('click', () => {
	const ticker = tickerInput?.value.toUpperCase();
	if (ticker) {
		startStream(ticker);
	}
});

// Start with default ticker on load
startStream(tickerInput?.value.toUpperCase() || 'AAPL');
