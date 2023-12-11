import express from 'express'
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

if (!process.env.MY_SECRET) {
  throw new Error("MY_SECRET is undefined or has length zero!")
}

app.use((req, res) => {
  res.status(404).send('Not Found');
});

const mySecret = process.env.MY_SECRET

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});