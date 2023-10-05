import * as wasm from "bullet_wasm";
import saveSvg from "./downloadBlob.js"
import base64Encode from "./base64.js"
import {logUSvg} from "./log.js"
//Web fonts are a rabbit hole, so defer for now.
// import {webFontLoader} from "./webfont.js"

import { Helper } from 'dxf'

const importCormorantFont = function(){
  fetch('/assets/Cormorant-Regular.ttf')
  .then(response => response.blob())
  .then(blob => blob.arrayBuffer())
  .then(arrayBuffer => new Uint8Array(arrayBuffer))
  .then(uint8Array => {
    wasm.js_add_font(uint8Array);
  });
}


const updateImageFromSvgStr = function(id, str){
 document.getElementById(id).src = "data:image/svg+xml;base64," + base64Encode(str);
}

const updateTextAreaFromSvgStr = function(id, str){
 document.getElementById(id).value = str;
}

const hideElementById = function(id){
  document.getElementById(id).style.display = "none";
}

const showElementById = function(id){
  document.getElementById(id).style.display = "block";
}

const getuSVGDims = function(usvgStr){
  let re = /<svg width="(\d*\.{1}\d*?mm)" height="(\d*\.{1}\d*?mm)"/;
  try{
    const m = usvgStr.match(re);
    if(m.length !== 3){
      throw new Error(`usvgDims: Wrong number of matches ${m.length}`);
    }
    return {width: Math.round(100* (Number(m[1].slice(0,-2)) + Number.EPSILON))/100 + "mm", height: Math.round(100* (Number(m[2].slice(0,-2)) + Number.EPSILON))/100 +"mm"};
  }catch(e){
    console.log(e);
  }
}

const toUsvgStr = function(inputSvgStr){
  //Render in meters
  // wasm.js_set_render_dpi(0.03937);
  const startTime = window.performance.now();
  const uSvgStrOrErr = wasm.js_process_svg_str_to_usvg_str(inputSvgStr);
  const currentTime = window.performance.now();
  console.log(`Start: ${startTime}, Current: ${currentTime} ElaspedTime: ${currentTime - startTime}`);

  if(uSvgStrOrErr.search("xmlns:usvg") == -1){
    throw Error(uSvgStrOrErr);
  }

  

  return uSvgStrOrErr;
}

const updateUsvgDimText = function(usvgStr){
  const {width, height} = getuSVGDims(usvgStr);

  document.getElementById("svgdims").innerHTML = `${width} x ${height}`;
}

const clearUsvgDimText = function(){
  document.getElementById("svgdims").innerHTML = "";
}


let logEntry;

const ENABLE_LOGGING = false;
const updateDemoPage = function(){
  //Set input image to inputSvgStr
  updateImageFromSvgStr('inputsvgimg', logEntry.originalSvg);
  //Set input text field to inputSvgStr
  updateTextAreaFromSvgStr('inputsvgtextarea', logEntry.originalSvg);

  //Parse logEntry.originalSvg to get uSvg
  try {
    logEntry.uSvg = toUsvgStr(logEntry.originalSvg);
  } catch (e) {
    logEntry.parserError = true;
    logEntry.errorDesc = e;
    console.error(e);
  }

  //Set output image to inputSvgStr
  updateImageFromSvgStr('usvgimg', logEntry.uSvg);
  //Set output text field to inputSvgStr
  updateTextAreaFromSvgStr('usvgtextarea', logEntry.uSvg);
  updateUsvgDimText(logEntry.uSvg);


  return (ENABLE_LOGGING ? logEntry.sendLogUSvg() : Promise.resolve())
  .then( (result) => console.log(result.logResponse) );
};

const saveUSvg =function(){
  if(logEntry.uSvg !== null){
    saveSvg(logEntry.uSvg, logEntry.fileName.slice(0,-4) + ".usvg.svg");
  }
}

const loadSvgFile = function(event){
  document.getElementById('image').checked = true;
  document.getElementById('source').checked = false;
  updateDisplayMode('image');

  const selectedFile = event.target.files[0];
  const reader = new FileReader();
  reader.onload = function(e){
    // console.log("where?");

    const fileExt = selectedFile.name.slice(-3).toLocaleLowerCase();
    
    logEntry = new logUSvg();
    
    if(fileExt === 'dxf'){
      const dxfParser = new Helper(e.target.result);
      logEntry.originalSvg = dxfParser.toSVG();
    }else{
      logEntry.originalSvg = e.target.result;
    }

    logEntry.fileName = selectedFile.name;
    // logEntry.originalSvg = fileStr;
    updateDemoPage();
  }

  reader.readAsText(selectedFile)
};

const updateDisplayMode = function(value){
  switch(value){
    case 'source':
    //hide images
    hideElementById('inputsvgimg');
    hideElementById('usvgimg');
    showElementById('inputsvgtextarea');
    showElementById('usvgtextarea');
    break;

    case 'image':
    //hide images
    showElementById('inputsvgimg');
    showElementById('usvgimg');
    hideElementById('inputsvgtextarea');
    hideElementById('usvgtextarea');
    break;
  }
}

const updateDisplayModeEvent = function(event){
  console.log(event.target.value);
  updateDisplayMode(event.target.value)
}

function init(){
  wasm.js_init_svg_parser();

  importCormorantFont();

  document.getElementById('svgfilepicker').addEventListener('change', loadSvgFile, false);
  document.getElementById('modeselectform').addEventListener('change', updateDisplayModeEvent, false);
  document.getElementById('usvgdownload').addEventListener('click', saveUSvg, false);
  
  //Modal event handlers - added bootstrap
  $("#errorReportModal").on("show.bs.modal", (e) => {
    if(logEntry){
      $("#filenameInput").val(logEntry.fileName);
    }
  })

  $("#errorReportModal").on("hidden.bs.modal", (e) => {
      $("#filenameInput").val("");
      $("#errorDescText").val("");
      $("#reporterName").val("");
  })

  $("#submitLogButton").on("click", (e) => {
    if(logEntry){
      logEntry.errorDec = $("#errorDescText").val();
      logEntry.errorReporter = $("#reporterName").val();

      $("#errorReportModal").modal('hide');
      
      return logEntry.sendLogUSvg()
      .then( (result) => console.log(result.logResponse)); 
    }
  })
}

window.onload = init();