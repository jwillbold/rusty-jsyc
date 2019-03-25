function transform(text) {
  // ...
  return text;
}

document.getElementById("mybutton").onclick = function() {

  var secret_text = document.getElementById("secret_ipt").value;
  console.log(secret_text);

  document.getElementById("secret_output").value = transform(secret_text);
};  





// XMLHttpRequest
