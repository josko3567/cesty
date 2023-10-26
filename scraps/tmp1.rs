
// extern "C" fn get_filerange(

//     tu: CXTranslationUnit, 
//     filename: &OsString ) -> CXSourceRange

// { unsafe {
//     let cfile = CString::new(filename.to_string_lossy().to_string()).unwrap();
//     let raw_cfile = cfile.into_raw();

//     let file = clang_getFile(tu, raw_cfile);
//     let fileSize = fs::metadata(filename).unwrap().len() as u32;

//     let _ = CString::from_raw(raw_cfile);
  
//     // get top/last location of the file
//     let topLoc  = clang_getLocationForOffset(tu, file, 0);
//     let lastLoc = clang_getLocationForOffset(tu, file, fileSize);
//     if clang_equalLocations(topLoc,  clang_getNullLocation()) != 0||
//        clang_equalLocations(lastLoc, clang_getNullLocation()) != 0 {
//     //   printf("cannot retrieve location\n");
//     //   exit(1).
//         return clang_getNullRange();
//     }

    
  
//     // make a range from locations
//     let range = clang_getRange(topLoc, lastLoc);
//     if clang_Range_isNull(range) != 0 {
//       return clang_getNullRange();
//     }
  
//     return range;

// }}


// fn _getTokenKindSpelling(kind: CXTokenKind) -> &'static str {

//     match kind {
//         CXToken_Punctuation  => {return "Punctuation"}
//         CXToken_Keyword      => {return "Keyword";}
//         CXToken_Identifier   => {return "Identifier";}
//         CXToken_Literal      => {return "Literal";}
//         CXToken_Comment      => {return "Comment";}
//         _ => {return "Unknown"}
//     }

// }

// fn show_all_tokens(
//     tu: CXTranslationUnit, 
//     tokens: *mut CXToken, 
//     numTokens: u32) 
// { unsafe {
//     println!("=== show tokens ===\n");
//     println!("NumTokens: {}\n", numTokens);
//     for i in 0..numTokens {
//         // let tokens =  
//       let token = *tokens.wrapping_add(i as usize);
//       let kind = clang_getTokenKind(token);
//       let spell = clang_getTokenSpelling(tu, token);
//       let loc = clang_getTokenLocation(tu, token);
  
//       let mut file: CXFile = null_mut();
//       let mut line: u32 = 0;
//       let mut column: u32 = 0;
//       let mut offset: u32 = 0;
//       clang_getFileLocation(loc, std::ptr::addr_of_mut!(file), null_mut(), null_mut(), null_mut());
//       let fileName = clang_getFileName(file);
  
//       println!("Token: {}\n", i);
//       println!(" Text: {:#?}\n", CStr::from_ptr(spell.data as *const i8).to_string_lossy().to_string());
//       println!(" Kind: {:#?}\n", _getTokenKindSpelling(kind));
//     //   println!(" Location: %s:%d:%d:%d\n",
//     //          clang_getCString(fileName), line, column, offset);
//       println!("\n");
  
//       clang_disposeString(fileName);
//       clang_disposeString(spell);
//     }
//   }}

    

    // let mut offset: u32 = 0;

    // let cpos = clang_getCursorLocation(ccur);
    // clang_getExpansionLocation(
    //     cpos,
    //     null_mut(),
    //     null_mut(),
    //     null_mut(),
    //     std::ptr::addr_of_mut!(offset) as *mut c_uint,
    // );
    // let name = clang_getCursorSpelling(ccur);
    // let ty = clang_getCursorKindSpelling(clang_getCursorKind(ccur));
    // let link = clang_getCursorLinkage(ccur);
    // let parent = clang_getCursorKindSpelling(clang_getCursorSemanticParent(ccur).kind);
    // println!("Kind: {} ^{}ˇ {}:{} Parent: {}", 
    //     CStr::from_ptr(ty.data as *const i8).to_string_lossy().to_string(),
    //     CStr::from_ptr(name.data as *const i8).to_string_lossy().to_string(),
    //     link,
    //     clang_getCursorLanguage(ccur),
    //     CStr::from_ptr(parent.data as *const i8).to_string_lossy().to_string(),
    // );

    // clang_getCursorLocation(ccur);
    // clang_getInstantiationLocation(location, file, line, column, offset);
    // clang_isStatement(kind)
    // println!(" {}", );

    // clang_disposeString(name);
    // clang_disposeString(ty);
    // clang_disposeString(parent);

    // offset_stack(OffsetStackOption::Push, Some((0,0)));


        // clang_tokenize(
        //     translation_unit, 
        //     get_filerange(translation_unit, &file.path), 
        //     std::ptr::addr_of_mut!(tokens),
        //     std::ptr::addr_of_mut!(n_tokens) as *mut c_uint,
        // );
        


        // show_all_tokens(translation_unit, tokens, n_tokens);