import express from 'express'
import pg from 'pg'
const { Pool } = pg

const app = express();
const port = 8871;

app.get('/start/', (req, res) => {
  res.json({'hello': 'deploy'});
});

app.get('/secret/:variable', (req, res) => {
  const variable = req.params.variable;

  if (variable === 'mysecreturl') {
    res.send(mySecret);
  } else {
    res.status(404).send('Not Found');
  }
});

app.get('/mode/', (req, res) => {
  res.send(environment);
});

app.get('/db/', async (req, res) => {
  const queryRes = await pool.query('SELECT $1::text as message', ['Hello deploy!'])
  const result = queryRes.rows[0].message

  res.send(result)
});

if (!process.env.MY_SECRET) {
  throw new Error("MY_SECRET is undefined or has length zero!")
}

app.use((req, res) => {
  res.status(404).send('Not Found');
});

const mySecret = process.env.MY_SECRET
// the environment of our app, by default it runs in production mode
const environment = process.env.APP_MODE ?? 'production'
// hellodeploy is our network name on which we can access the container
// if we are in local development we just connect on localhost
const host = environment === 'localdev' ? '127.0.0.1' : `hellodeploy-db-${environment}`

// We set default values in both cases
const dbPass = process.env.POSTGRES_PASSWORD ?? 'postpost'
const dbPort = parseInt(process.env.POSTGRES_PORT) ?? 5432

const pool = new Pool({
  host,
  user: 'postgres',
  port: dbPort,
  password: dbPass,
})

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});
