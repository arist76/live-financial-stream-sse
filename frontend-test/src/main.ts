'''import './style.css';

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <h1>Live Financial Stream (SSE)</h1>
    <div id="data-container"></div>
  </div>
`;

const dataContainer = document.querySelector<HTMLDivElement>('#data-container');

const eventSource = new EventSource('http://127.0.0.1:3000');

eventSource.onmessage = (event) => {
  const quote = JSON.parse(event.data);
  const quoteElement = document.createElement('div');
  quoteElement.innerHTML = `<strong>${quote.symbol}</strong>: $${quote.price.toFixed(2)} @ ${new Date(quote.timestamp * 1000).toLocaleTimeString()}`;
  dataContainer?.appendChild(quoteElement);
};

eventSource.onerror = (err) => {
  console.error("EventSource failed:", err);
};
''