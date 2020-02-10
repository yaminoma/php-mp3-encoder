<?php

if(isset($_POST['encode'])){
    $file = $_FILES['audio_file']['tmp_name'];
    $handle = fopen($file, "rb"); 
    $fsize = filesize($file); 
    $value = fread($handle, $fsize); 
    $ffi = loadLibrary();
    $length = \strlen($value);
    $buffer = FFI::new(FFI::arrayType($ffi->type('uint8_t'), [$length]));
    FFI::memcpy($buffer, $value, $length);
    $bufferPtr = FFI::cast($ffi->type('uint8_t*'), $buffer);
    $output = $ffi->encode($bufferPtr, $length);
    $fname = 'convert.mp3';
    header('Content-Type: application/force-download');
    header('Content-Length: '.filesize($output));
    header('Content-disposition: attachment; filename="'.$fname.'"');
    readfile($output);
    unlink($output);
}

function loadLibrary() :\FFI
{
    $signature = "const char* encode(uint8_t *data, uint32_t length);";
    return \FFI::cdef( $signature, __DIR__ . "/../target/release/libmp3encoder.dylib");
}
?>
<!doctype html>
<html lang="ja">
<head>
    <meta charset="utf-8">
    <title>test</title>
</head>
<body>
<form method="post" enctype="multipart/form-data">
<input type="file" name="audio_file"><br>
<input type="submit" value="upload" name="encode">
</form>
</body>
</html>
