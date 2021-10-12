## Interprether

Webapp that exposes a live feed of Ethereum transactions whose input data can be decoded to UTF-8 strings. Available here: [https://interprether.tommasopifferi.com](https://interprether.tommasopifferi.com)

The app can run on [every browser](https://caniuse.com/wasm) that supports WebAssembly.

### Develop

Clone the repository:

```bash
$ git clone git@github.com:neslinesli93/interprether.git
```

Copy the correct `.env` file:

```bash
$ cp .env.development .env
```

Finally:

```bash
$ docker-compose up
```

This command will start several containers:

- a web page available at [http://localhost:8080](http://localhost:8080)
- a simple http backend that exposes an endpoint that return transactions' data
- a scanner that looks for transactions in new mined blockks
- a redis store
- a local [GETH](https://geth.ethereum.org/) instance
