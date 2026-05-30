import { setupServer } from "msw/node";

// A shared MSW server; individual tests register handlers via `server.use(...)`.
export const server = setupServer();
