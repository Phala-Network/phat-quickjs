import "./polyfill-textencoding";
import "./polyfill-streams";
import "./polyfill-fetch";
import "./polyfill-url";
import "./polyfill-xhr";
import "./wapo";
import "./polyfill-abortcontroller";
import "./polyfill-blob";

import { Headers } from "headers-polyfill";
globalThis.Headers = Headers;
