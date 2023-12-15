import "./polyfill-textencoding";
import "./polyfill-streams";
import "./polyfill-fetch";
import "./polyfill-url";
import "./polyfill-xhr";
import "./sidevm";
import "./polyfill-abortcontroller";

import { Headers } from "headers-polyfill";
globalThis.Headers = Headers;
