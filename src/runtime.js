(globalThis => {
  globalThis.console = {
    log: (...args) => {
      Deno.core.opAsync("op_log", args.map(arg => JSON.stringify(arg)).join(" "), false);
    },
  };
})(globalThis);
