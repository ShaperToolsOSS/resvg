import axios from 'axios';

const thisAxios = axios.create({
  timeout: 120000
});

//Scans svg for web font urls, retrieves any found font files

//For each font family, find url if any in CSS
//Add better parsing of css stylesheets later
export const webFontLoader = function(svgString){
  //Find font families
  const fontFamilies = svgString.match(/(?<=font-family\s*=\s*").+(?=\")/);

  // console.log(fontFamilies);

  const fontUrls = fontFamilies.map(ff => {
    
    //Get first part of font name in lower case
    let fontName = ff.match(/(?<=\s*)\S*/)[0].toLowerCase();

    //Find url that includes this fontname
    let urls = svgString.match(/(?<=url\s*\(["']).*(?=['"])/).map(u => u.toLowerCase()).filter(u => u.includes(fontName));

    return {fontFamily: ff, urls};
  });
  //for each URL, download file
  //Take first url for now

  const fontPromises = [];

  fontUrls.forEach(fu => {
    if(fu.urls.length > 0){
      fontPromises.push(
        thisAxios.get(fu.urls[0])
        .then(response => {
          debugger;
          fu.fontData = response.data;
          return fu;
        })
        .catch(error => {
          console.log(error);
        })
      );
    }
  });

  return Promise.all(fontPromises);
}
