const fs = require('fs');
const {execSync} = require('child_process');

execSync('npm run build');

const removeDir = path => {
  if (fs.existsSync(path)) {
    const files = fs.readdirSync(path)

    if (files.length > 0) {
      files.forEach(function(filename) {
        if (fs.statSync(path + "/" + filename).isDirectory()) {
          removeDir(path + "/" + filename)
        } else {
          fs.unlinkSync(path + "/" + filename)
        }
      })
      fs.rmdirSync(path)
    } else {
      fs.rmdirSync(path)
    }
  }
}

if (fs.existsSync('www')){
  removeDir('www');
}
fs.mkdirSync('www')
fs.mkdirSync('www/static')
fs.createReadStream('public/favicon.png').pipe(fs.createWriteStream('www/static/favicon.png'));
fs.createReadStream('public/global.css').pipe(fs.createWriteStream('www/static/global.css'));
fs.createReadStream('public/build/bundle.js').pipe(fs.createWriteStream('www/static/bundle.js'));
fs.createReadStream('public/build/bundle.css').pipe(fs.createWriteStream('www/static/bundle.css'));
fs.createReadStream('public/build/bundle.js.map').pipe(fs.createWriteStream('www/static/bundle.js.map'));

let data = fs.readFileSync('public/index.html', 'utf8');
data = data.replace('/favicon.png', '/static/favicon.png');
data = data.replace('/global.css', '/static/global.css');
data = data.replace('/manifest.json', '/static/manifest.json');
data = data.replace('/build/bundle.js', '/static/bundle.js');
data = data.replace('/build/bundle.css', '/static/bundle.css');
data = data.replace('/service-worker.js', '/static/service-worker.js');
fs.writeFileSync('www/index.html', data);


let data2 = fs.readFileSync('public/manifest.json', 'utf8');
data2 = data2.replace('/favicon.png', '/static/favicon.png');
fs.writeFileSync('www/static/manifest.json', data2);

let data3 = fs.readFileSync('public/service-worker.js', 'utf8');
data3 = data3.replace('/favicon.png', '/static/favicon.png');
data3 = data3.replace('/global.css', '/static/global.css');
data3 = data3.replace('/manifest.json', '/static/manifest.json');
data3 = data3.replace('/build/bundle.js', '/static/bundle.js');
data3 = data3.replace('/build/bundle.css', '/static/bundle.css');
data3 = data3.replace('/build/bundle.js.map', '/static/bundle.js.map');
data3 = data3.replace('/service-worker.js', '/static/service-worker.js');
fs.writeFileSync('www/static/service-worker.js', data3);