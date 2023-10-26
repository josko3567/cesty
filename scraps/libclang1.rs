// nsafe{

//         let file = CString::new("/home/josko-k/nonstd/asigraph/src/agcont.c")
//             .expect("CString::failed!");
//         let name = file.as_ptr();
//         let clang = clang_createIndex(0, 0);
//         let unit = clang_parseTranslationUnit(
//             clang, 
//             name, 
//             null(), 
//             0,
//             null_mut(),
//             0,
//             CXTranslationUnit_None
//             // | CXTranslationUnit_SkipFunctionBodies
//         );

//         /// there ARE FLAGS!!!!!!!!!!!!!!!!!!1
        

//         extern "C" fn print_names(current: CXCursor, parent: CXCursor, data: CXClientData) -> i32 {
//             unsafe {
//                 let loc = clang_getCursorLocation( current );

//                 if clang_Location_isFromMainFile( loc ) != 0 {
//                     // return CXChildVisit_Continue;
//                     let current_display_name = clang_getCursorDisplayName(current);
//                     if !current_display_name.data.is_null(){
//                         let a = clang_getCursorKindSpelling(current.kind);
//                         if current.kind == CXCursor_FunctionDecl{  
//                             let res = clang_getTypeSpelling(clang_getResultType(clang_getCursorType(current)));
//                             // clang_getTypeSpelling
//                             println!("#! {}\n", CStr::from_ptr(res.data as *const i8).to_string_lossy());
//                             println!("#! {}\n", CStr::from_ptr(current_display_name.data as *const i8).to_string_lossy() );
//                             // let b = clang_getTypeSpelling(clang_getPointeeType(clang_getCursorType(current)));
//                             // println!("--- {}\n", CStr::from_ptr(b.data as *const i8).to_string_lossy())
//                             clang_disposeString(res);
                            
//                         } 
//                         // else {
//                         //     println!("{}: {}\n", 
//                         //         CStr::from_ptr(a.data as *const i8).to_string_lossy(),
//                         //         CStr::from_ptr(current_display_name.data as *const i8).to_string_lossy()
//                         //     ); 
//                         // }
//                         clang_disposeString(a);
//                         clang_disposeString(current_display_name);
//                     }
//                 }
//                 return CXChildVisit_Continue;
//             }
//         }
        
//         // clang_getCursorAvailability(cursor)
//         if unit.is_null() {
//             println!("Failed to init TranslationUnit for clang");
//             exit(1);
//         }

//         let cursor = clang_sys::clang_getTranslationUnitCursor(unit);

//         clang_visitChildren(
//             cursor,
//             print_names,
//             null_mut()
//         );

//         let _directory = CString::new("/home/josko-k/nonstd/asigraph")
//             .expect("CString::failed!");

//         let error: *mut i32 = null_mut();

//         // clang_CompilationDatabase_fromDirectory(_directory.as_ptr(), error);

//         clang_disposeTranslationUnit(unit);
//         clang_disposeIndex(clang);
//         // clang_free(cursor);

// }