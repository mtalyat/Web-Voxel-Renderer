import init, { run_program } from "../pkg/project.js";

let loadTextResource = function(url, callback)
{
    let request = new XMLHttpRequest();
    request.open('GET', url + '?dc=' + Math.random(), true);
    request.onloadend = function()
    {
        if(request.status < 200 || request.status > 299)
        {
            callback(false, 'Request status invalid for loading file: ' + url);
        } else
        {
            callback(true, request.responseText);
        }
    };
    request.send();
}

const CANVAS_ID = "canvas";

async function run() {
  await init();
  const canvasSize = [canvas.width, canvas.height];
  
  loadTextResource('../src/vertex.glsl', (vpass, vtext) => {
    if (!vpass)
    {
        alert('Fatal error loading vertex shader.');
        console.error(vtext);
        return;
    }

    loadTextResource('../src/fragment.glsl', (fpass, ftext) => {
        if (!fpass)
        {
            alert('Fatal error loading fragment shader.');
            console.error(ftext);
            return;
        }

        // got both shaders, now run the program
        run_program(CANVAS_ID, canvasSize, vtext, ftext);
    });
  });
}

let resize_canvas = function()
{
    const canvas = document.querySelector('canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

window.addEventListener('resize', () => {
    resize_canvas();
});

resize_canvas();
run();