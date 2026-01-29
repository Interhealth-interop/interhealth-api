<div><h1>InterHealth API</h1></div>

[![Typescript](https://img.shields.io/badge/typescript-4.6.4-blue?logo=typescript&logoColor=white)](https://www.npmjs.com/package/typescript "Go to TypeScript on NPM")
[![RabbitMQ](https://img.shields.io/badge/RabbitMQ-3.9.14-orange?logo=RabbitMQ&logoColor=white)](https://www.rabbitmq.com/ "Go to RabbitMQ website")
![Build](https://img.shields.io/badge/build-passing-brightgreen?logo=appveyor&logoColor=white)

<br>

&#xa0;

## :dart: About the Project

InterHealth API is a project that automates the scheduled import of partner data into InterHealth systems. It ensures seamless and timely data integration, enhancing data accuracy and operational efficiency.

&#xa0;

## :rocket: Technologies

The following tools were used in this project:

- [Typescript](https://www.npmjs.com/package/typescript)

- [RabbitMQ](https://www.rabbitmq.com/)

&#xa0;

## :white_check_mark: Requirements

Before starting :checkered_flag:, you need to have [Git](https://git-scm.com) and [Node](https://nodejs.org/en/) installed.

&#xa0;

## :checkered_flag: Starting

#### <a name="token"></a> Environment Variables:

- `APP_PORT` {number, optional} {default: 3000} - The port on which the API will run.
- `MONGO_URL` {string, required} - MongoDB connection string.
- `JWT_SECRET` {string, optional} {default: default-secret-change-in-production} - Secret key for JWT token generation.
- `MAX_CONCURRENT_JOBS` {number, optional} {default: 5} - Maximum number of concurrent synchronization jobs that can run in parallel.
- `RUST_LOG` {string, optional} {default: debug} - The log level for Rust logging.

&#xa0;

#### <a name="token"></a> Installing dependencies

```bash
$ npm install
```

&#xa0;

## :question: Guidelines

&#xa0;

A project implemented by the InterHealth Team at InterHealth Solutions. :star:

## Build Project:

```bash
$ cargo build
```

<a href="#top">Back to top</a>

POST /sync/init
└─ Se job pausado → retoma
└─ Se job running → erro 409
└─ Se não existe → cria novo

POST /sync/jobs/:id/pause
└─ Pausa job running

POST /sync/jobs/:id/resume
└─ Retoma job pausado (continua de onde parou)
└─ Só aceita status Paused

POST /sync/jobs/:id/restart
└─ Reexecuta qualquer job
└─ Se Paused → continua
└─ Se Completed/Failed → recomeça do zero