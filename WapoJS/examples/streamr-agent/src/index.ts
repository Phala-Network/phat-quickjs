// Run a Streamr node right inside your JS app
const StreamrClient = require("@streamr/sdk");

const streamr = new StreamrClient();

// Subscribe to a stream of messages
streamr.subscribe("streams.dimo.eth/firehose/weather", (msg: any) => {
  // Handle incoming messages
  console.log(`Incoming msg: ${JSON.stringify(msg)}`);
});
