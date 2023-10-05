function saveFileBlob(content, filename, contentType){
  
  //contentType = "image/svg+xml" for svg files
  const linkElement = document.createElement('a');

  try {
    const blob = new Blob([content], { type: contentType });
    const url = window.URL.createObjectURL(blob);

    linkElement.setAttribute('href', url);
    linkElement.setAttribute('download', filename);

    const clickEvent = new MouseEvent('click', {
      view: window,
      bubbles: true,
      cancelable: false
    });
    linkElement.dispatchEvent(clickEvent);

  } catch (ex) {
    console.log(new Error(ex));
  }
};

export default function(content, filename){
  saveFileBlob(content, filename, 'image/svg+xml');
};
