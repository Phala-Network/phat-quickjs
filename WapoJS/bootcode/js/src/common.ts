import "./wapo";
import "./polyfill-dom-basic";
import "./polyfill-textencoding";
import "./polyfill-streams";
import "./polyfill-fetch";
import "./polyfill-url";
import "./polyfill-abortcontroller";
import "./polyfill-abortsignal-more";
import "./polyfill-blob";
import "./polyfill-websocket";

import { Headers } from "headers-polyfill";
globalThis.Headers = Headers;
