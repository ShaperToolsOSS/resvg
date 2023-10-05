import axios from 'axios';

const thisAxios = axios.create({
  timeout: 120000
});

export class logUSvg{
  constructor({
    fileName = 'foo',
    parserError = false,
    errorDesc = '',
    errorReporter = '',
    originalSvg = '',
    uSvg = '',
   }={}){
    this.fileName = fileName;
    this.parserError = parserError;
    this.errorDesc = errorDesc;
    this.errorReporter = errorReporter;
    this.originalSvg = originalSvg;
    this.uSvg = uSvg;
  }

  sendLogUSvg(){
    return thisAxios.post('api/logSvgParse', this)
    .then(logRes => logRes.data)
    .catch(error => {
      console.log(error);
    });
  }
}

