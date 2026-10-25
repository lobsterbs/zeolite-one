const http = require('http');
const server = http.createServer((req, res) => {
  res.statusCode = 404;
  res.setHeader('content-type', 'text/plain');
  res.end('gone\n');
});
const port = process.env.PORT || 3000;
server.listen(port, () => console.log('stub on ' + port));
