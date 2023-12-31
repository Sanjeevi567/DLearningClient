use aws_apis::{
    create_celebrity_single_pdf, create_detect_face_image_pdf, create_polly_voice_info_pdf,
    create_text_only_pdf, load_credential_from_env, CredentInitialize, PollyOps, RekognitionOps,
    S3Ops, TranscribeOps, TranslateOps,
};
use colored::Colorize;
use dotenv::dotenv;
use image::{self, GenericImageView, Rgba};
use image_compressor::FolderCompressor;
use imageproc::drawing::draw_text_mut;
use inquire::{
    ui::{Attributes, RenderConfig, StyleSheet, Styled},
    Confirm, Select, Text,
};
use regex::Regex;
use rusttype::{Font, Scale};
use serde_json::{Map, Value};
use std::env::var;
use std::fs::{create_dir, read_dir, read_to_string, remove_dir_all, File, OpenOptions};
use std::io::{Read, Write};

#[tokio::main]
async fn main() {
    inquire::set_global_render_config(global_render_config());
    let operations: Vec<&str> = vec![
        "Verify the Credentials\n",
        "Print Credentials Information\n",
        "Amazon Polly Operations\n",
        "Amazon Rekognition Operations\n",
        "Amazon Translate\n",
        "Amazon Transcribe\n",
        "Quit the application\n",
    ];
    //Initial dummy credentials
    let mut credential = CredentInitialize::default();
    let mut sdk_config = credential.build();
    let mut rekognition_ops: RekognitionOps = RekognitionOps::build(&sdk_config);
    let mut transcribe_ops = TranscribeOps::build(&sdk_config);
    let mut polly_ops: PollyOps = PollyOps::build(&sdk_config);
    let mut s3_ops: S3Ops = S3Ops::build(&sdk_config);
    let mut translate_ops = TranslateOps::build(&sdk_config);

    'main: loop {
        let choice = Select::new("Select the option to execute the operation\n", operations.clone())
            .with_help_message("Don't enclose data in quotation marks or add spaces around it in any operations,\nexcept when working with template data.")
            .with_page_size(7)
            .prompt()
            .unwrap();
        match choice {
            "Verify the Credentials\n" => {
                let choices = Confirm::new("Load the credentials from the configuration file or from environment variables\n")
                          .with_placeholder("Use 'Yes' to load from the environment and 'No' to load from environment variables\n")
                          .with_help_message("Without proper credentials, no operations can be executed successfully")
                          .prompt()
                          .unwrap();
                match choices {
                    true => {
                        let (credentials, region) = load_credential_from_env().await;
                        credential.update(
                            credentials.access_key_id(),
                            credentials.secret_access_key(),
                            region.as_deref(),
                        );
                        sdk_config = credential.build();
                        s3_ops = S3Ops::build(&sdk_config);
                        polly_ops = PollyOps::build(&sdk_config);
                        rekognition_ops = RekognitionOps::build(&sdk_config);
                        transcribe_ops = TranscribeOps::build(&sdk_config);
                        translate_ops = TranslateOps::build(&sdk_config);
                        println!("{}\n","Please verify the credentials by printing the credential information before proceeding with any operations".yellow().bold());
                    }
                    false => {
                        dotenv().ok();
                        let access_key = var("AWS_ACCESS_KEY_ID")
                        .expect("Ensure that the 'AWS_ACCESS_KEY_ID' environment variable is set, and its value is provided by AWS\n");
                        let secret_key = var("AWS_SECRET_ACCESS_KEY")
                        .expect("Ensure that the 'AWS_SECRET_ACCESS_KEY' environment variable is set, and its value is provided by AWS\n");
                        let region = var("AWS_DEFAULT_REGION")
                        .expect("Ensure that the 'AWS_DEFAULT_REGION' environment variable is set, and its value is provided by AWS\n");
                        credential.update(&access_key, &secret_key, Some(&region));
                        sdk_config = credential.build();
                        s3_ops = S3Ops::build(&sdk_config);
                        polly_ops = PollyOps::build(&sdk_config);
                        rekognition_ops = RekognitionOps::build(&sdk_config);
                        transcribe_ops = TranscribeOps::build(&sdk_config);
                        translate_ops = TranslateOps::build(&sdk_config);
                        println!("{}\n","Please verify the credentials by printing the credential information before proceeding with any operations".yellow().bold());
                    }
                }
            }
            "Print Credentials Information\n" => {
                let confirm =
                    Confirm::new("Are you sure you want to print credential information?\n")
                        .with_formatter(&|str| format!(".....{str}.....\n"))
                        .with_placeholder("Type 'Yes' to view the credentials, or 'No' to not view the credentials\n")
                        .with_help_message("This is solely for verification purposes")
                        .prompt()
                        .unwrap();

                match confirm {
                    true => {
                        println!("{}\n","Here is your credential informations".yellow().bold());
                        credential.print_credentials();
                    }
                    false => {
                        println!("{}\n", "Sure...".green().bold())
                    }
                }
            }
            "Amazon Polly Operations\n" => {
                let polly_operations = vec![
                    "Start the Speech Synthesis Task\n",
                    "Get the Speech Synthesis Results\n",
                    "List all Speech Synthesis Tasks\n",
                    "Generate All Voices Audio in MP3\n",
                    "Obtain voice information from Amazon Polly\n",
                    "Return to the Main Menu\n",
                ];

                loop {
                    let polly_choices = Select::new(
                        "Select the option to execute the operation\n",
                        polly_operations.clone(),
                    )
                    .with_help_message("Do not enclose it with quotation marks or add spaces")
                    .with_vim_mode(true)
                    .with_page_size(6)
                    .prompt()
                    .unwrap();
                    match polly_choices {
                        "Start the Speech Synthesis Task\n" => {
                            let possible_engines =
                                "Possible Engine Values are:\n    'standard'\n    'neural'\n";
                            let engine_name =
                                Text::new("Select the speech generation engine name\n")
                                    .with_placeholder(possible_engines)
                                    .with_formatter(&|str| format!(".....{str}.....\n"))
                                    .prompt()
                                    .unwrap();
                            match engine_name.is_empty() {
                                false => {
                                    let (voice_ids, lang_codes) =
                                        polly_ops.get_voice_info_given_engine(&engine_name).await;
                                    let mut vec_of_voice_ids = Vec::new();
                                    voice_ids.into_iter().for_each(|voice_id| {
                                        if let Some(voiceid) = voice_id {
                                            vec_of_voice_ids.push(voiceid.as_str().to_owned());
                                        }
                                    });
                                    let available_voiceid_specified_engine = format!("Voice ID's for the specified engine: {engine_name}\n{:?}\n",vec_of_voice_ids.join(" | "));
                                    let voice_id = Text::new(
                                        "Select the voice for audio generation\n",
                                    )
                                    .with_placeholder(&available_voiceid_specified_engine)
                                    .with_formatter(&|str| format!(".....{str}.....\n"))
                                    .with_help_message(
                                        "Click here https://tinyurl.com/3wzknfnw to learn more",
                                    )
                                    .prompt()
                                    .unwrap();
                                    let mut vec_of_lang_codes = Vec::new();
                                    lang_codes.into_iter().for_each(|lang_code| {
                                        if let Some(langcode) = lang_code {
                                            vec_of_lang_codes.push(langcode.as_str().to_string());
                                        }
                                    });
                                    let available_langcodes_specified_engine = format!("Language codes for the specified engine: {engine_name}\n{:?}\n",vec_of_lang_codes.join(" | "));

                                    let language_code = Text::new("Select the audio language\n")
                                        .with_placeholder(&available_langcodes_specified_engine)
                                        .with_formatter(&|str| format!(".....{str}.....\n"))
                                        .with_help_message(
                                            "Click here https://tinyurl.com/27f3zbhd to learn more",
                                        )
                                        .prompt()
                                        .unwrap();

                                    let possible_text_types = "    ssml  |     text";
                                    let text_type = Text::new("Please provide the text format of the content for which you would like to synthesize audio\n")
                         .with_placeholder(possible_text_types)
                         .with_formatter(&|str| format!(".....{str}.....\n"))
                         .with_help_message("Click here https://tinyurl.com/zyuwuvhp to learn more")
                         .prompt()
                         .unwrap();
                                    let text_to_generate_speech_path = Text::new("Please specify the path of the text file for which you would like audio generation\n")
                         .with_placeholder("The format of the text content is determined by the preceding selections\n")
                         .with_help_message("Click here https://tinyurl.com/ynjmpur3 to Learn more")
                         .with_formatter(&|str| format!(".....{str}.....\n"))
                         .prompt()
                         .unwrap();
                                    let valid_formats = "  json |   mp3 |   ogg_vorbis |   pcm";
                                    let audio_output_format = Text::new("Please select the output format for the generated speech content\n")
                        .with_placeholder(valid_formats)
                        .with_formatter(&|str| format!(".....{str}.....\n"))
                        .prompt()
                        .unwrap();
                                    let available_buckets = format!(
                                        "Available Buckets in your account:\n{:#?}\n",
                                        s3_ops.get_buckets().await
                                    );
                                    let bucket_name = Text::new("Amazon S3 bucket name to which the output file will be saved\n")
                         .with_placeholder(&available_buckets)
                         .with_formatter(&|str| format!(".....{str}.....\n"))
                         .with_help_message("The chosen bucket name should be available in different regions to enable multi region access")
                         .prompt()
                         .unwrap();
                                    match (
                                        voice_id.is_empty(),
                                        language_code.is_empty(),
                                        text_type.is_empty(),
                                        text_to_generate_speech_path.is_empty(),
                                        audio_output_format.is_empty(),
                                        bucket_name.is_empty(),
                                    ) {
                                        (false, false, false, false, false, false) => {
                                            let mut speech_text_data = OpenOptions::new()
                                                .read(true)
                                                .write(true)
                                                .open(&text_to_generate_speech_path)
                                                .expect(
                                                    "Error Opening the file path you specified\n",
                                                );
                                            let mut text_to_generate_speech = String::new();
                                            speech_text_data
                                                .read_to_string(&mut text_to_generate_speech)
                                                .expect("Error while reading data\n");

                                            polly_ops
                                                .start_speech_synthesise_task(
                                                    &engine_name,
                                                    &voice_id,
                                                    &language_code,
                                                    &text_type,
                                                    &text_to_generate_speech,
                                                    &audio_output_format,
                                                    &bucket_name,
                                                )
                                                .await;
                                        }
                                        _ => println!(
                                            "{}\n",
                                            "Fields can't be left empty".red().bold()
                                        ),
                                    }
                                }
                                true => {
                                    println!("{}\n", "Engine name can't be left empty".red().bold())
                                }
                            }
                        }
                        "Generate All Voices Audio in MP3\n" => {
                            let possible_engines =
                                "Possible Engine Values are:\n '    standard'\n'    neural'\n";
                            let engine_name =
                                Text::new("Select the engine name for generating all the voices using this engine\n")
                                    .with_placeholder(possible_engines)
                                    .with_formatter(&|input| format!("Received Engine Is: '{input}'\n"))
                                    .prompt()
                                    .unwrap();
                            match engine_name.is_empty() {
                                false => {
                                    let (_, lang_codes) =
                                        polly_ops.get_voice_info_given_engine(&engine_name).await;
                                    let mut vec_of_lang_codes = Vec::new();
                                    lang_codes.iter().for_each(|lang_code| {
                                        if let Some(langcode) = lang_code {
                                            vec_of_lang_codes.push(langcode.as_str().to_string());
                                        }
                                    });
                                    let available_langcodes_specified_engine = format!("Language codes for the specified engine: {engine_name}\n{:?}\n",vec_of_lang_codes.join(" | "));

                                    let language_code = Text::new("Select the Audio language\n")
                                        .with_placeholder(&available_langcodes_specified_engine)
                                        .with_formatter(&|input| {
                                            format!("Received Language Code Is: '{input}'\n")
                                        })
                                        .with_help_message(
                                            "Click here https://tinyurl.com/27f3zbhd to learn more",
                                        )
                                        .prompt()
                                        .unwrap();
                                    let voice_counts = lang_codes.iter().count();
                                    let placeholder_info =format!("A total of '{voice_counts}' voices will be generated for the SSML text you provide");
                                    let text_to_generate_speech_path = Text::new("Please specify the path to the SSML text file\n")
                                        .with_placeholder(&placeholder_info)
                                        .with_help_message("Click here https://tinyurl.com/bdf5uhce to download the sample SSML text file")
                                        .with_formatter(&|input| format!("Received SSML Text Path Is: '{input}'"))
                                        .prompt()
                                        .unwrap();
                                    let path_prefix = Text::new("Enter the path prefix under which you want to save the content in the current path\n")
                                        .with_placeholder("For example, 'neural/' or 'standard/ \n")
                                        .with_formatter(&|input|format!("Received Path Prefix Is: {input}\n"))
                                        .with_help_message("The directory will be created anew. Ensure that no directory with the same name as the one you specify already exists, and with each run, select a different directory prefix")
                                        .prompt()
                                        .unwrap();
                                    match (
                                        language_code.is_empty(),
                                        text_to_generate_speech_path.is_empty(),
                                        path_prefix.is_empty(),
                                    ) {
                                        (false, false, false) => {
                                            std::fs::create_dir(&path_prefix)
                                                .expect("Error while creating directory prefix\n");
                                            let mut read_data = OpenOptions::new()
                                                .read(true)
                                                .write(true)
                                                .open(&text_to_generate_speech_path)
                                                .expect("Error while opening the ssml file path\n");
                                            let mut text_data = String::new();
                                            read_data
                                                .read_to_string(&mut text_data)
                                                .expect("Error while reading to string\n");
                                            polly_ops
                                                .generate_all_available_voices_in_mp3(
                                                    &text_data,
                                                    &language_code,
                                                    &engine_name,
                                                    &path_prefix,
                                                )
                                                .await;
                                        }
                                        _ => println!(
                                            "{}\n",
                                            "Fields should not be left empty".red().bold()
                                        ),
                                    }
                                }
                                true => println!("{}\n", "Engine Name can't be empty".red().bold()),
                            }
                        }
                        "Get the Speech Synthesis Results\n" => {
                            let task_id = Text::new("To obtain speech results, enter the task ID\n")
                                .with_placeholder("Task ID was generated when calling the StartSpeechSynthesisTask REST API or\nis available in the current directory if you chose the 'Start the speech synthesis task' option\n")
                                .with_formatter(&|str| format!(".....{str}.....\n"))
                                .prompt()
                                .unwrap();
                            match task_id.is_empty() {
                                false => {
                                    let info =
                                        polly_ops.get_speech_synthesis_result(&task_id).await;
                                    if let Some(synthesise_info) = info {
                                        let status = synthesise_info.get_task_status();
                                        let engine = synthesise_info.get_engine();
                                        let output_uri = synthesise_info.get_output_uri();
                                        let output_format = synthesise_info.get_output_format();
                                        let text_type = synthesise_info.get_text_type();
                                        let voice_id = synthesise_info.get_voice_id();
                                        let language_code = synthesise_info.get_language_code();
                                        let status_reason =
                                            synthesise_info.get_task_status_reason();
                                        match status.unwrap_or("No Status Is Available".into()) {
                                            "scheduled" => {
                                                println!("{}\n","Current task status is 'scheduled,' so please try again after some time.\nThe result is only available when the status is 'completed'".yellow().bold());
                                            }
                                            "inProgress" => {
                                                println!("{}\n","Current task status is 'in progress,' so please try again after some time.\nThe result is only available when the status is 'completed'".yellow().bold());
                                            }
                                            "completed" => {
                                                if let (
                                                    Some(status),
                                                    Some(engine),
                                                    Some(uri),
                                                    Some(format),
                                                    Some(text),
                                                    Some(voice),
                                                    Some(code),
                                                ) = (
                                                    status,
                                                    engine,
                                                    output_uri,
                                                    output_format,
                                                    text_type,
                                                    voice_id,
                                                    language_code,
                                                ) {
                                                    let colored_status = status.green().bold();
                                                    let colored_engine = engine.green().bold();
                                                    let colored_uri = uri.green().bold();
                                                    let colored_format = format.green().bold();
                                                    let colored_type = text.green().bold();
                                                    let colored_voiceid = voice.green().bold();
                                                    let colored_code = code.green().bold();
                                                    println!("Task Status: {colored_status}");
                                                    println!("Engine Name: {colored_engine}");
                                                    println!("Output Format of the synthesized audio: {colored_format}");
                                                    println!("Voice ID of the synthesized audio: {colored_voiceid}");
                                                    println!(
                                                        "Text type of synthesized audio: {colored_type}"
                                                    );
                                                    println!("Language Code for the synthesized audio: {colored_code}");
                                                    println!(
                                                        "URL for the synthesized audio: {colored_uri}"
                                                    );
                                                    let mut file = OpenOptions::new()
                                                        .create(true)
                                                        .read(true)
                                                        .write(true)
                                                        .open("audio_uri.txt")
                                                        .unwrap();
                                                    let uri_data = format!(
                                                        "URL for the synthesized audio: {uri}\n"
                                                    );
                                                    file.write_all(uri_data.as_bytes())
                                                        .expect("Error while writting...");
                                                    println!(
                                                        "{}\n",
                                                        "The URL is written to the current directory.".green().bold()
                                                    );
                                                    println!("{}","The bucket can't be accessed right away; you have to make it public or only accessible in the web console".yellow().bold());
                                                    println!("{}\n","Alternatively, you can make the object accessible using the 'Modify Object Visibility' option in the S3 menu or download them using the 'Download Object from bucket' option".yellow().bold());
                                                }
                                            }
                                            "failed" => {
                                                println!("{}", "The task has failed".red().bold());
                                                println!("{}", "Reason, if any".yellow().bold());
                                                println!(
                                                    "{}\n",
                                                    status_reason
                                                        .unwrap_or("No reason is available")
                                                );
                                            }
                                            _ => println!("Shoudn't reach"),
                                        }
                                    }
                                }
                                true => println!("{}\n", "Task ID can't be empty".red().bold()),
                            }
                        }

                        "List all Speech Synthesis Tasks\n" => {
                            polly_ops.list_synthesise_speech().await;
                        }

                        "Obtain voice information from Amazon Polly\n" => {
                            let info = polly_ops.describe_voices().await;
                            info.iter()
                .take(3)
                .for_each(|voice_info|{
                 if let (Some(gender),Some(voiceid),Some(lang_code),Some(lang_name),Some(voice_name),Some(engines)) = 
                  (voice_info.get_gender(),voice_info.get_voiceid(),voice_info.get_language_code(),
                 voice_info.get_language_name(),voice_info.get_voice_name(),voice_info.get_supported_engines())
                  {
                     println!("Gender: {}\nVoiceId: {}\nLanguageCode: {}\nLanguage Name: {}\nVoice Name: {}",
                     gender.green().bold(),
                     voiceid.green().bold(),
                     lang_code.green().bold(),
                     lang_name.green().bold(),
                     voice_name.green().bold()
                    );
                    engines.iter()
                    .for_each(|engine|{
                     println!("Supported Engine: {}\n",engine.green().bold());
                    });
                 }
                });

                            let mut file = OpenOptions::new()
                                .create(true)
                                .read(true)
                                .write(true)
                                .open("voices_info.txt")
                                .unwrap();
                            let colored_file_name = "'voices_info.txt'".green().bold();
                            let msg = format!("There is a lot more information available, so it only displays the first three pieces of voice information.\n\nAll the voice information is saved to the current directory as {colored_file_name} instead of cluttering the command-line window");
                            println!("{}\n", msg);
                            let headers = vec![
                                "Gender of Voice",
                                "Voice ID",
                                "Language Code",
                                "Language Name",
                                "Voice Name",
                                "Supported Engine",
                            ];
                            let mut values = Vec::new();
                            info.into_iter()
                .for_each(|voice_info|{
                 if let (Some(gender),Some(voiceid),Some(lang_code),Some(lang_name),Some(voice_name),Some(engines)) = 
                  (voice_info.get_gender(),voice_info.get_voiceid(),voice_info.get_language_code(),
                 voice_info.get_language_name(),voice_info.get_voice_name(),voice_info.get_supported_engines())
                  {
                     let data = format!("Gender:           {}\nVoiceId:          {}\nLanguageCode:     {}\nLanguage Name:    {}\nVoice Name:       {}\nSupported Engine: {}\n\n",
                     gender,
                     voiceid,
                     lang_code,
                     lang_name,
                     voice_name,
                     engines.iter().map(|str|*str).collect::<Vec<&str>>().join("")
                 );
                  values.push(gender.to_string());
                  values.push(voiceid.to_string());
                  values.push(lang_code.to_string());
                  values.push(lang_name.to_string());
                  values.push(voice_name.to_string());
                  values.push(engines.into_iter().collect::<String>());
                  file.write_all(data.as_bytes())
                  .expect("Error while writing data...")
                 }
                });
                            create_polly_voice_info_pdf(headers, values);

                            println!(
                                "{}\n",
                                "Content is writen to current directory".green().bold()
                            );
                        }

                        "Return to the Main Menu\n" => continue 'main,

                        _ => println!("Never Reach"),
                    }
                }
            }
            "Amazon Rekognition Operations\n" => {
                let rekog_ops = vec![
                    "Recognize a Celebrity\n",
                    "Upload Images to an S3 Bucket\n",
                    "Recognize Multiple Celebrities\n",
                    "Face detection\n",
                    "Text detection\n",
                    "Upload Modified Images to an S3 bucket\n",
                    "Write images with facial details obtained from Rekognition's 'DetectFaces' feature\n",
                    "Start a face detection task\n",
                    "Get face detection results\n",
                    "Start a text detection task\n",
                    "Get text detection results\n",
                    "Return to the Main Menu\n",
                ];
                loop {
                    let rekog_choices = Select::new(
                        "Select the option to execute the operation\n",
                        rekog_ops.clone(),
                    )
                    .with_page_size(12)
                    .prompt()
                    .unwrap();
                    match rekog_choices {
                        "Upload Modified Images to an S3 bucket\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new("Please enter the bucket name where you'd like to store images for the Face Detection Tasks\n")
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|input| format!("Received Bucket Name: {input}\n"))
                            .with_help_message("Ensure that the chosen bucket and region match")
                            .prompt()
                            .unwrap();
                        
                        let local_path_prefix = Text::new("Provide the local path prefix where your all images are stored\n")
                            .with_placeholder("Please Note that the images will be resized to 800x600 pixels, but the original images on your computer will remain unchanged\n")
                            .with_formatter(&|input| format!("Received Local Path Prefix: {input}\n"))
                            .with_help_message("These images should be in either '.jpg' or '.png' format")
                            .prompt()
                            .unwrap();
                        match (bucket_name.is_empty(),local_path_prefix.is_empty()){
                            (false,false) => {
                                let get_objects =
                                s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                            let available_objects = format!(
                                "Available keys and path prefix in {bucket_name}\n{:#?}\n",
                                get_objects
                            );
                            let bucket_path_prefix = Text::new("Select a prefix for the bucket where the images will be saved\n")
                            .with_placeholder(&available_objects)
                            .with_help_message("For example, you can use 'face_images/' or 'images/'")
                            .with_formatter(&|input| format!("Received Bucket Path Prefix: {input}"))
                            .prompt()
                            .unwrap();
                            let create_temp_dir = "modified/";
                            create_dir(create_temp_dir).expect("Error while creating modified/ temp directory\n");
                            let entries = read_dir(&local_path_prefix).expect("Error while reading directory\n");
                            for entry in entries {
                                let entry = entry.expect("Error while reading entries in the directory");
                                match entry.file_name().to_str() {
                                    Some(image_name) => {
                                        let local_image_file_name = format!("{local_path_prefix}/{image_name}");
                                        let image = image::open(&local_image_file_name)
                                            .expect("Error while reading the image from path\n");
                                        let image = image.resize_to_fill(800, 600, image::imageops::FilterType::Gaussian);
                                        let path_and_file_name = format!("{create_temp_dir}{image_name}");
                                        image
                                            .save(&path_and_file_name)
                                            .expect("Error while writing image file\n");
                                        let have_slash_and_dot_pattern =
                                            Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
                                        let file_name: Vec<&str> = have_slash_and_dot_pattern
                                            .find_iter(&path_and_file_name)
                                            .map(|string| string.as_str())
                                            .collect();
                                        println!("{}\n", file_name.join(""));
                                        let key_name = format!("{bucket_path_prefix}{}", file_name.join(""));
                                        s3_ops
                                            .upload_content_to_a_bucket(&bucket_name, &path_and_file_name, &key_name)
                                            .await;
                                    }
                                    None => println!("{}\n", "No file is found".red().bold()),
                                }
                            }
                            remove_dir_all("modified/")
                                .expect("Error while deleing modified temp directory the directory\n");
                            }
                            _ => println!("{}\n","No fields can be empty".red().bold())
                        }
                    
                        }
                        "Write images with facial details obtained from Rekognition's 'DetectFaces' feature\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new("Please enter the name of the bucket where the images are stored\n")
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|input| format!("Received Bucket Name Is: {input}"))
                            .prompt()
                            .unwrap();
                        
                        match bucket_name.is_empty(){
                            false => {
                                let get_objects =
                                s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                            let available_objects = format!(
                                "Available keys and path prefix in {bucket_name}\n{:#?}\n",
                                get_objects
                            );
                            println!("{}\n","This operation assumes a single face in the image".yellow().bold());
                            let bucket_path_prefix = Text::new("Enter the path prefix within the bucket where the images are stored\n")
                            .with_formatter(&|input| format!("Received Bucket Path Prefix Is: {input}"))
                            .with_placeholder(&available_objects)
                            .with_help_message("Please ensure that there is no 'face_details_images' directory in the current path where the application is running")
                            .prompt()
                            .unwrap();
                        println!("");
                        match std::fs::remove_dir_all("face_details_images/"){
                            Ok(_) => println!("{}\n","face_details_images/ in the current directory have been deleted".red().bold()),
                            Err(_) => println!("{}\n","Sure no face_details_images directory exist in the current directory".yellow().bold())
                        };
                        let mut file = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .open("Face_details.txt")
                        .unwrap();
                
                    let entries = s3_ops.list_objects_given_prefix(&bucket_name, &bucket_path_prefix).await;
                    let face_details_images = "face_details_images/";
                    create_dir(face_details_images)
                        .expect("Error while creating face_details_image/ directory for writting images\n");
                    let local_path_prefix = "read_images/";
                    create_dir(local_path_prefix).expect("Error while creating read_images/ temp directory\n");
                
                    for image_path in entries.iter(){
                        s3_ops
                            .download_content_from_bcuket(&bucket_name, &image_path, Some(local_path_prefix), false)
                            .await;
                        let outputs = rekognition_ops.detect_faces(&image_path, &bucket_name).await;
                        for mut face_detail in outputs.into_iter() {
                            let have_slash_and_dot_pattern =
                                Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
                            let image_name: Vec<&str> = have_slash_and_dot_pattern
                                .find_iter(&image_path)
                                .map(|string| string.as_str())
                                .collect();
                            println!(
                                "{} {}\n",
                                "Details of image".yellow().bold(),
                                image_name.join("").green().bold()
                            );
                            if let (
                                (Some(smile), Some(smile_confidence)),
                                (Some(gender), Some(gender_confidence)),
                                (Some(age_range), Some(age_confidence)),
                                (Some(beard), Some(beard_confidence)),
                                (Some(width), Some(height), Some(left), Some(top)),
                            ) = (
                                face_detail.smile(),
                                face_detail.gender(),
                                face_detail.age_range(),
                                face_detail.beard(),
                                face_detail.bounding_box(),
                            ) {
                                let details = vec![
                                    format!("Details of image: {}\n", image_name.join("")),
                                    format!("Gender: {gender}, with a confidence level of {gender_confidence}\n"),
                                    format!(
                                        "Age Range: {age_range}, with a confidence level of {age_confidence}\n"
                                    ),
                                    format!("Beard: {beard}, with a confidence level of {beard_confidence}\n"),
                                    format!("Smile: {smile}, with a confidence level of {smile_confidence}\n"),
                                    format!(
                                        "Bounding Box Details: Width: {}, Height: {}, Left: {}, Top: {}\n\n",
                                        width, height, left, top
                                    ),
                                ];
                                for detail in details {
                                    file.write_all(detail.as_bytes()).unwrap();
                                }
                                //drawing code
                                let read_image_path = format!("{local_path_prefix}{}", image_name.join(""));
                                let image = image::open(&read_image_path)
                                    .expect("Error while reading the image from path\n");
                                let image_dimension = image.dimensions();
                                let mut new_or_old_image = if image_dimension == (800,600){
                                    image
                                }else{
                                    let image_new = image.resize_to_fill(800, 600, image::imageops::FilterType::Gaussian);
                                    image_new
                                };
                                let scale = Scale::uniform(30.0);
                                let color = Rgba([255u8, 0u8, 0u8, 127u8]);
                                let data_ = include_bytes!("./assets/font.ttf");
                                let font =
                                    Font::try_from_bytes(data_).expect("Error Getting Font Bytes");
                                let gender = format!("Gender: {gender}");
                                let age = format!("Age: {age_range}");
                                let beard = format!("Beard: {beard}");
                                let smile = format!("Smile: {smile}");
                
                                draw_text_mut(&mut new_or_old_image, color, 0, 0, scale, &font, &gender);
                                draw_text_mut(&mut new_or_old_image, color, 0, 50, scale, &font, &age);
                                draw_text_mut(&mut new_or_old_image, color, 0, 100, scale, &font, &beard);
                                draw_text_mut(&mut new_or_old_image, color, 0, 150, scale, &font, &smile);
                
                                let modified_image_path_name =
                                    format!("{face_details_images}{}", image_name.join(""));
                                    new_or_old_image
                                    .save(&modified_image_path_name)
                                    .expect("Error while writing Image file\n");
                            }
                        }
                    }
                
                    remove_dir_all("read_images/").expect("Error while Deleting read_images/ temp dir");
                    create_dir("compressed_images/").expect("Error while creating compressed_images/ dir");
                    let mut compressor = FolderCompressor::new("face_details_images/", "compressed_images/");
                    compressor.set_thread_count(8);
                    compressor.set_delelte_origin(true);
                    compressor.compress().expect("Error while compressing Images\n");
                    create_detect_face_image_pdf(&bucket_name, &bucket_path_prefix);
                    println!(
                        "{}\n",
                        "Face details are written to the current directory with the name 'face_details.txt'"
                            .green()
                            .bold()
                    );
                    println!(
                        "{}\n",
                        "Images with face details are saved in the 'face_details_images' directory within the current path".green().bold()
                    );
                            }
                            _ => println!("{}\n","Neither Bucket Name nor Bucket Path Prefix Can't be Empty".red().bold())
                        }
                    
                        }
                        "Face detection\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let blob = "https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/primitives/struct.Blob.html";
                            let help_message = format!("S3 buckets are employed instead of {blob} types for processing face images");
                            let bucket_name = Text::new(
                                "Select the bucket name where the face image is stored\n",
                            )
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .with_help_message(&help_message)
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let get_objects =
                                        s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                    let available_objects = format!(
                                        "Available keys in {bucket_name}\n{:#?}\n",
                                        get_objects
                                    );
                                    let object = Text::new("Please input the key or path of the face image within the chosen bucket or copy it from the placeholder information\n")
                                        .with_placeholder(&available_objects)
                                        .with_formatter(&|str| format!(".....{str}.....\n"))
                                        .with_help_message("Don't put quotation marks around the key when pasting")
                                        .prompt()
                                        .unwrap();
                                    match object.is_empty() {
                                        false => {
                                            let face_info = rekognition_ops
                                                .detect_faces(&object, &bucket_name)
                                                .await;
                                            let mut file = OpenOptions::new()
                                            .create(true)
                                            .read(true)
                                            .write(true)
                                            .open("FaceDetail.txt")
                                            .expect("Error while creating file\n");
                                            face_info.into_iter().for_each(|mut facedetails| {
                                                let gender = facedetails.gender();
                                                let age = facedetails.age_range();
                                                let smile = facedetails.smile();
                                                let beard = facedetails.beard();
                                                let bounding_box = facedetails.bounding_box();

                                                if let (
                                                    (Some(gender), Some(gender_confidence)),
                                                    (Some(age), Some(age_confidence)),
                                                    (Some(smile), Some(smile_confidence)),
                                                    (Some(beard), Some(beard_confidence)),
                                                    (Some(width),Some(height),Some(left),Some(top))
                                                ) = (gender, age, smile, beard,bounding_box)
                                                {
                                                    let buf = format!("Gender: {gender} and Confidence Level: {gender_confidence}\nAge Range:\nLowest Prediction Age: {age} and Highest Prediction Age: {age_confidence}\nSmile: {smile} and Confidence Levle: {smile_confidence}\nBeard: {beard} and Confidence: {beard_confidence}\nBounding Box Details:\nWidth: {width}, Height: {height}, Left: {left},Top: {top}");
                                                    file.write_all(buf.as_bytes()).unwrap();

                                                }
                                            });
                                            match std::fs::File::open("FaceDetail.txt") {
                                                Ok(_) => println!(
                                                    "{}\n",
                                                    "The text file, containing the text details, has been successfully written to the current directory"
                                                        .green()
                                                        .bold()
                                                ),
                                                Err(_) => println!("{}\n", "Error while writing File".red().bold()),
                                            }
                                        }
                                        true => {
                                            println!(
                                                "{}\n",
                                                "key/object name can't be empty".red().bold()
                                            )
                                        }
                                    }
                                }
                                true => {
                                    println!("{}\n", "Bucket name can't be empty".red().bold())
                                }
                            }
                        }
                        "Text detection\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let blob = "https://docs.rs/aws-sdk-rekognition/latest/aws_sdk_rekognition/primitives/struct.Blob.html";
                            let help_message = format!("S3 buckets are employed instead of {blob} types for processing texts");
                            let bucket_name = Text::new(
                                "Please select the bucket name where the image is stored, which contains the text within it\n",
                            )
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .with_help_message(&help_message)
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let get_objects =
                                        s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                    let available_objects = format!(
                                        "Available keys in {bucket_name}\n{:#?}\n",
                                        get_objects
                                    );
                                    let object = Text::new("Input the key or path of the image from the chosen bucket, or copy it from the placeholder information\n")
                                        .with_placeholder(&available_objects)
                                        .with_formatter(&|str| format!(".....{str}.....\n"))
                                        .with_help_message("Don't put quotation marks around the key when pasting")
                                        .prompt()
                                        .unwrap();
                                    match object.is_empty() {
                                        false => {
                                            let text_info = rekognition_ops
                                                .detect_texts(&bucket_name, &object)
                                                .await;
                                            let mut texts_only = Vec::new();
                                            let mut file = OpenOptions::new()
                                            .create(true)
                                            .read(true)
                                            .write(true)
                                            .open("Texts.txt")
                                            .expect("Error while creating file\n");
                                            text_info.into_iter().for_each(|mut textdetails| {
                                                let texts = textdetails.detected_text();
                                                let text_type = textdetails.text_type();
                                                let confidence = textdetails.confidence();

                                                if let (
                                                    Some(text),
                                                    Some(text_type),
                                                    Some(confidence),
                                                ) = (texts, text_type, confidence)
                                                {
                                                    let buf = format!("Detected Text: {text}\nText Type: {text_type}\nText Confidence: {confidence}\n");
                                                    file.write_all(buf.as_bytes()).unwrap();
                                                    texts_only.push(text);
                                                }
                                            });
                                            match std::fs::File::open("Texts.txt") {
                                                Ok(_) => println!(
                                                    "{}\n",
                                                    "The text file, containing the text details, has been successfully written to the current directory"
                                                        .green()
                                                        .bold()
                                                ),
                                                Err(_) => println!("{}\n", "Error while writing File".red().bold()),
                                            }
                                            create_text_only_pdf(texts_only);
                                        }
                                        true => {
                                            println!(
                                                "{}\n",
                                                "key/object name can't be empty".red().bold()
                                            )
                                        }
                                    }
                                }
                                true => {
                                    println!("{}\n", "Bucket name can't be empty".red().bold())
                                }
                            }
                        }
                        "Start a face detection task\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);

                            let help_message =
                                format!("S3 buckets are used to store face and videos");
                            let bucket_name = Text::new(
                                "Select the bucket name where the face video is stored\n",
                            )
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .with_help_message(&help_message)
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let get_objects =
                                        s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                    let available_objects = format!(
                                        "Available keys in {bucket_name}\n{:#?}\n",
                                        get_objects
                                    );
                                    let key_video_name = Text::new("Please input the key or path of the face video within the chosen bucket or copy it from the placeholder information\n")
                                        .with_placeholder(&available_objects)
                                        .with_formatter(&|str| format!(".....{str}.....\n"))
                                        .with_help_message("Don't put quotation marks around the key when pasting")
                                        .prompt()
                                        .unwrap();
                                    match key_video_name.is_empty() {
                                        false => {
                                            rekognition_ops
                                                .start_face_detection_task(
                                                    &bucket_name,
                                                    &key_video_name,
                                                )
                                                .await;
                                        }
                                        true => {
                                            println!(
                                                "{}\n",
                                                "key/object name can't be empty".red().bold()
                                            )
                                        }
                                    }
                                }
                                true => {
                                    println!("{}\n", "Bucket name can't be empty".red().bold())
                                }
                            }
                        }
                        "Get face detection results\n" => {
                            let job_id = Text::new("To obtain the results of the face detection task, please enter the job ID\n")
                                .with_placeholder("The job ID was generated when you initiated the start face detection task\n")
                                .with_formatter(&|str| format!("......{str}......\n"))
                                .prompt()
                                .unwrap();

                            match job_id.is_empty() {
                                false => {
                                    let mut face_info =
                                        rekognition_ops.get_face_detection_results(&job_id).await;
                                    let job_status = face_info.job_status();
                                    let status_message = face_info.status_message();
                                    if let Some(job_status) = job_status {
                                        match &*job_status {
                                            "IN_PROGRESS" => {
                                                println!("The job status is currently marked as '{}' which means no output is generated until the status changes to '{}'","IN_PROGRESS".green().bold(),"SUCCEEDED".yellow().bold());
                                                println!("{}\n","Please check back after some time to obtain the results of the face detection process".yellow().bold());
                                            }
                                            "SUCCEEDED" => {
                                                println!("It appears that the job status is now '{}', and the output processing has begun\n","SUCCEEDED".green().bold());
                                                face_info
                                                    .write_face_detection_results_as_text_and_pdf();
                                            }
                                            "FAILED" => {
                                                println!("It appears that the job status is '{}'. For some reason, the face detection task has failed","FAILED".green().bold());
                                                println!("{}\n","Please try again by restarting the face detection process. Good luck!\n".yellow().bold());
                                            }
                                            _ => {}
                                        }
                                    }
                                    if let Some(status_msg) = status_message {
                                        println!(
                                            "Status Message is: {}\n",
                                            status_msg.green().bold()
                                        );
                                    }
                                }
                                true => {
                                    println!("{}\n", "Job ID can't be empty".red().bold())
                                }
                            }
                        }
                        "Start a text detection task\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);

                            let help_message =
                                format!("S3 buckets are used to store text and videos");
                            let bucket_name = Text::new(
                                "Select the bucket name where the text video is stored\n",
                            )
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .with_help_message(&help_message)
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let get_objects =
                                        s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                    let available_objects = format!(
                                        "Available keys in {bucket_name}\n{:#?}\n",
                                        get_objects
                                    );
                                    let key_text_name = Text::new("Please input the key or path of the text video within the chosen bucket or copy it from the placeholder information\n")
                                        .with_placeholder(&available_objects)
                                        .with_formatter(&|str| format!(".....{str}.....\n"))
                                        .with_help_message("Don't put quotation marks around the key when pasting")
                                        .prompt()
                                        .unwrap();
                                    match key_text_name.is_empty() {
                                        false => {
                                            rekognition_ops
                                                .start_text_detection_task(
                                                    &bucket_name,
                                                    &key_text_name,
                                                )
                                                .await;
                                            println!("");
                                        }
                                        true => {
                                            println!(
                                                "{}\n",
                                                "key/object name can't be empty".red().bold()
                                            )
                                        }
                                    }
                                }
                                true => {
                                    println!("{}\n", "Bucket name can't be empty".red().bold())
                                }
                            }
                        }
                        "Get text detection results\n" => {
                            let job_id = Text::new("To obtain the results of the text detection task, please enter the job ID\n")
                            .with_placeholder("The job ID was generated when you initiated the start text detection task\n")
                            .with_formatter(&|str| format!("......{str}......"))
                            .prompt()
                            .unwrap();
                            match job_id.is_empty() {
                                false => {
                                    let mut text_results =
                                        rekognition_ops.get_text_detection_results(&job_id).await;
                                    let job_status = text_results.job_status();
                                    let status_message = text_results.status_message();
                                    if let Some(job_status) = job_status {
                                        match &*job_status {
                                            "IN_PROGRESS" => {
                                                println!("The job status is currently marked as '{}' which means no output is generated until the status changes to '{}'","IN_PROGRESS".green().bold(),"SUCCEEDED".yellow().bold());
                                                println!("{}\n","Please check back after some time to obtain the results of the face detection process".yellow().bold());
                                            }
                                            "SUCCEEDED" => {
                                                println!("It appears that the job status is now '{}', and the output processing has begun\n","SUCCEEDED".green().bold());
                                                text_results
                                                    .write_text_detection_results_as_text_and_pdf();
                                            }
                                            "FAILED" => {
                                                println!("It appears that the job status is '{}'. For some reason, the face detection task has failed","FAILED".green().bold());
                                                println!("{}\n","Please try again by restarting the face detection process. Good luck!\n".yellow().bold());
                                            }
                                            _ => {}
                                        }
                                    }
                                    if let Some(status_msg) = status_message {
                                        println!(
                                            "Status Message is: {}\n",
                                            status_msg.green().bold()
                                        );
                                    }
                                }
                                true => println!("{}\n", "Job ID can't be empty".red().bold()),
                            }
                        }
                        "Recognize a Celebrity\n" => {
                            let local_or_s3 = Confirm::new(
                                "Either you want to provide the local or S3 location for the celebrity image\n",
                            )
                            .with_placeholder("Type 'Yes' to load from the local location, or type 'No' to provide the S3 object location\n")
                            .with_formatter(&|input| format!("Received Response Is: {input}\n"))
                            .with_help_message("Only the celebrity image should be used, and the results are based on AWS APIs")
                            .prompt()
                            .unwrap();
                            match local_or_s3{
                                true => {
                                    let celebrity_image_path = Text::new(
                                        "Please provide the path to the celebrity image or paste it here without quotation marks\n",
                                    )
                                    .with_placeholder("The image should be in '.JPG' or '.PNG' format; no other formats are supported.\n")
                                    .with_formatter(&|input| format!("Received Path Is: {input}\n"))
                                    .prompt()
                                    .unwrap();
                                    rekognition_ops.recognize_celebrities(Some(&celebrity_image_path),None,None).await;
                                    println!("{}\n","Please ensure that you rename or move the text file if you decide to execute this option again".yellow().bold());
                                    println!("{}\n","Please roll up to view the complete output of the operation".yellow().bold());
                                }
                                false => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new(
                                "Please enter the bucket name where the celebrity images are stored\n",
                            )
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|input| format!("Received Path Is: {input}\n"))
                            .with_help_message("Ensure that the bucket and the region are the same as where you are making the request")
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let get_objects =
                                s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                            let available_objects = format!(
                                "Available keys and path prefix in {bucket_name}\n{:#?}\n",
                                get_objects
                            );
                            let bucket_key = Text::new("Please provide the key path for the actual celebrity image\n")
                            .with_placeholder(&available_objects)
                            .with_help_message("For example: 'celebrityimages/ar rahman.jpg' or 'robert downey jr.png'")
                            .with_formatter(&|input| format!("Received Bucket Key: {input}\n"))
                            .prompt()
                            .unwrap();
                            match bucket_key.is_empty(){
                                 false => {
                                    rekognition_ops.recognize_celebrities(None,Some(&bucket_name),Some(&bucket_key)).await;
                                    println!("{}\n","The image has also been downloaded to the current directory".green().bold());
                                    println!("{}\n","Please roll up to view the complete output of the operation".yellow().bold());
                                 }
                                 true => println!("{}\n","Bucket Key/object can't empty".red().bold())
                            }
                                   
                                }
                                true => println!("{}\n", "Bucket name can't be empty".red().bold()),
                            }
                                }
                            }
                        }
                        "Upload Images to an S3 Bucket\n" => {
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new("Please enter the bucket name where you'd like to store images for the 'Recognize Multiple Celebrities' option\n")
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|input| format!("Received Bucket Name: {input}\n"))
                            .with_help_message("Ensure that the chosen bucket and region match")
                            .prompt()
                            .unwrap();
                        
                        let local_path_prefix = Text::new("Provide the local path prefix under which all your celebrity JPG or PNG images are stored\n")
                            .with_placeholder(r#"Eg: 'CelebrityImages/', 'E:\CelebrityImages'\n"#)
                            .with_formatter(&|input| format!("Received Local Path Prefix: {input}\n"))
                            .with_help_message("These images should be in either '.jpg' or '.png' format")
                            .prompt()
                            .unwrap();
                        match (bucket_name.is_empty(),local_path_prefix.is_empty()){
                            (false,false) => {
                                let get_objects =
                                s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                            let available_objects = format!(
                                "Available keys and path prefix in {bucket_name}\n{:#?}\n",
                                get_objects
                            );
                            let bucket_path_prefix = Text::new("Select a prefix for the bucket under which all your uploaded images will be saved\n")
                            .with_placeholder(&available_objects)
                            .with_help_message("For example, you can use 'celebrityimages/' or 'images/'")
                            .with_formatter(&|input| format!("Received Bucket Path Prefix: {input}\n"))
                            .prompt()
                            .unwrap();
                            match bucket_path_prefix.is_empty(){
                                 false =>{
                                    let entries = read_dir(&local_path_prefix).expect("Error while reading directory You Specified\n");
                            for entry in entries {
                                let entry = entry.expect("Error while reading entries in the directory");
                                match entry.file_name().to_str() {
                                    Some(image_name) => {
                                        let local_image_file_name = format!("{local_path_prefix}/{image_name}");
                                        let have_slash_and_dot_pattern =
                                            Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
                                        let file_name: Vec<&str> = have_slash_and_dot_pattern
                                            .find_iter(&local_image_file_name)
                                            .map(|string| string.as_str())
                                            .collect();
                                        //println!("{}\n", file_name.join(""));
                                        let key_name = format!("{bucket_path_prefix}{}", file_name.join(""));
                                        s3_ops
                                            .upload_content_to_a_bucket(&bucket_name, &local_image_file_name, &key_name)
                                            .await;
                                        
                                    }
                                    None => println!("{}\n", "No file is found".red().bold()),
                                }
                                println!("Please provide '{}' as the prefix for 'Recognize Multiple Celebrities'\nwhen asking for the bucket path key or prefix to retrieve images under this prefix",&bucket_path_prefix.green().bold());
                            }
                                 }
                                 true => println!("{}\n","Bucket path prefix can't be empty".red().bold())
                            }
                        }
                        _ => println!("{}\n","Both bucket name and local directory path Can't Empty".red().bold())
                    }
                }
                        "Recognize Multiple Celebrities\n" => {
                            println!("");
                            println!("{}\n","Please make sure that there is no 'DownloadedImages/' directory in the location where your application is running,\nas it will be deleted if it exists when you choose s3 location".yellow().bold());
                            let local_or_s3 = Confirm::new(
                                "Either you want to provide the local or S3 location for the celebrity Images\n",
                            )
                            .with_placeholder("Type 'Yes' to load from the local location, or type 'No' to provide the S3 object prefix\n")
                            .with_formatter(&|input| format!("Received Response Is: {input}\n"))
                            .with_help_message("Only the celebrity images should be used, and the results are based on AWS APIs")
                            .prompt()
                            .unwrap();
                        match  local_or_s3{
                            true => {
                                let celebrity_images_dir = Text::new(
                                    "Please provide the directory where the celebrity images exist in JPG or PNG formats\n",
                                )
                                .with_placeholder(r#"Eg: 'Celebrity Images/' or 'E:\New folder\CelebrityImages'\n"#)
                                .with_help_message("The images should be in '.JPG' or '.PNG' format; no other formats are supported")
                                .with_formatter(&|input| format!("Received Path Is: {input}\n"))
                                .prompt()
                                .unwrap();
                                match celebrity_images_dir.is_empty(){
                                    false => {
                                        create_celebrity_single_pdf(Some(&celebrity_images_dir),None,None).await;
                                    }
                                    true => println!("{}\n","The Field Celebrity Images Dir can't be empty".red().bold())
                                }
            
                            }
                            false => {
                                match remove_dir_all("DownloadedImages/"){
                                    Ok(_) => println!(""),
                                    Err(_) => println!("")
                                }
                                let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new("Enter the bucket name where the celebrity images are stored\n")
                            .with_placeholder(&available_buckets)
                            .with_formatter(&|input| format!("Received Bucket Name: {input}\n"))
                            .with_help_message("The bucket name is what you used to upload images using the 'Upload Images to an S3 Bucket' option")
                            .prompt()
                            .unwrap();
                            match bucket_name.is_empty(){
                                false => {
                            let get_objects =
                                    s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                let available_objects = format!(
                                    "Available keys and path prefix in {bucket_name}\n{:#?}\n",
                                    get_objects
                                );
                            let bucket_path_prefix = Text::new("Please enter the bucket path prefix under which the celebrity images can be retrieved\n")
                            .with_placeholder(&available_objects)
                            .with_help_message("For example, 'celebrityimages/' or something similar if that's how you named the prefix")
                            .with_formatter(&|input| format!("Received Bucket Path Prefix: {input}\n"))
                            .prompt()
                            .unwrap();
                                    let entries = s3_ops
                                    .list_objects_given_prefix(&bucket_name, &bucket_path_prefix)
                                    .await;
                                create_celebrity_single_pdf(None, Some(entries), Some(&bucket_name)).await;
                                }
                                true => println!("{}\n","Bucket Name can't be empty".red().bold())
                            }
                              
                            }
                        }
                       
                            
                        }
                        "Return to the Main Menu\n" => continue 'main,
                        _ => println!("Never Reach"),
                    }
                }
            }
            "Amazon Translate\n" => {
                let translate_opss = vec![
                    "Get Language Info\n",
                    "Translate Text\n",
                    "Translate Document\n",
                    "Start Text Translation Job\n",
                    "Describe Text Translation Job\n",
                    "List Text Translation Jobs\n",
                    "Return to the Main Menu\n",
                ];
                loop {
                    let translate_choices = Select::new(
                        "Select the option to execute the operation\n",
                        translate_opss.clone(),
                    )
                    .with_page_size(7)
                    .with_help_message(
                        "Only six of the most commonly used APIs from the translation service are currently being utilized",
                    )
                    .prompt()
                    .unwrap();
                    match translate_choices {
                        "Get Language Info\n" => {
                            translate_ops.list_languages(true).await;
                        }
                        "Translate Text\n" => {
                            let path_to_the_text_data = Text::new("Please provide the path to the text file for which you would like the translation\n")
                                .with_placeholder("The provided text should only be in plain text format; no other formats should be used in this option\n")
                                .with_formatter(&|input| format!("Received Text Path: {}\n", input))
                                .prompt()
                                .unwrap();
                            let (lang_names, lang_codes) =
                                translate_ops.list_languages(false).await;
                            let mut placeholder_info = Vec::new();
                            for (lang_code, lang_name) in
                                lang_codes.into_iter().zip(lang_names.into_iter())
                            {
                                let format_lang_code_and_name =
                                    format!("{}: {}", lang_name,lang_code);
                                placeholder_info.push(format_lang_code_and_name);
                            }
                            let target_lang_code = Text::new("Provide the target language code for which you want to receive the translation in return\n")
                                .with_placeholder(&placeholder_info.join(" | "))
                                .with_help_message("Copy the target language code from the placeholder that corresponds to the language name for which you want a translation")
                                .with_formatter(&|input| {
                                    format!("Received Target Language Code to Translate: {}\n", input)
                                })
                                .prompt()
                                .unwrap();
                            match (
                                path_to_the_text_data.is_empty(),
                                target_lang_code.is_empty(),
                            ) {
                                (false, false) => {
                                    translate_ops
                                        .translate_text(&path_to_the_text_data, &target_lang_code)
                                        .await;
                                }
                                _ => println!(
                                    "{}\n",
                                    "Ensure that no fields are left empty".red().bold()
                                ),
                            }
                        }
                        "Translate Document\n" => {
                            let document_type = Text::new("Please specify the type of document for which you need translation\n")
                            .with_placeholder("Valid Document Types\n   'Plain' - Plain Text Document\n   'Word' -Word Document\n   'Html' -Html Document\n")
                            .with_formatter(&|input| format!("Received Document Type: {}\n", input))
                            .with_help_message("Copy the document type inside the single quotes depending on the document you provide next")
                            .prompt()
                            .unwrap();
                            let path_to_the_document = Text::new("Please provide the path to the Document file for which you would like the translation\n")
                            .with_placeholder("The format of the document should reflect the type of document you provided above\n")
                            .with_formatter(&|input| format!("Received Document Path: {}\n", input))
                            .prompt()
                            .unwrap();
                            let (lang_names, lang_codes) =
                                translate_ops.list_languages(false).await;
                            let mut placeholder_info = Vec::new();
                            for (lang_code, lang_name) in
                                lang_codes.into_iter().zip(lang_names.into_iter())
                            {
                                let format_lang_code_and_name =
                                    format!("{}: {}",lang_name,lang_code);
                                placeholder_info.push(format_lang_code_and_name);
                            }
                            let target_lang_code = Text::new("Provide the target language code for which you want to receive the translation in return\n")
                            .with_placeholder(&placeholder_info.join(" | "))
                            .with_help_message("Copy the target language code from the placeholder that corresponds to the language name for which you want a translation")
                            .with_formatter(&|input| {
                                format!("Received Target Language Code to Translate: {}\n", input)
                            })
                            .prompt()
                            .unwrap();
                            match (
                                document_type.is_empty(),
                                path_to_the_document.is_empty(),
                                target_lang_code.is_empty(),
                            ) {
                                (false, false, false) => {
                                    translate_ops
                                        .translate_document(
                                            &document_type,
                                            &path_to_the_document,
                                            &target_lang_code,
                                        )
                                        .await;
                                }
                                _ => println!(
                                    "{}\n",
                                    "Ensure that no fields are left empty".red().bold()
                                ),
                            }
                        }
                        "Start Text Translation Job\n" => {
                            let job_name = Text::new("Please choose a unique job name that describes the translation task\n")
                            .with_placeholder("You are responsible for selecting unique descriptive job name\n")
                            .with_formatter(&|input| format!("Received Job Name: {}\n", input))
                            .prompt()
                            .unwrap();
                            let document_type = Text::new("Specify the document type you have in the S3 bucket\n")
                            .with_placeholder("Valid Document Types\n   'Plain' - Plain Text File\n   'Word' -Word Document\n   'Html' -Html Document\n   'Ppt' -PPT Document\n   'Xlsx' -XLSX Document\n   'Xlf' -Lossless XLF Document\n")
                            .with_formatter(&|input| format!("Received Document Type: {}\n", input))
                            .with_help_message("You have the option to batch-translate files of the same format. If you need to translate different formats, please start a new job")
                            .prompt()
                            .unwrap();
                            let (lang_names, lang_codes) =
                                translate_ops.list_languages(false).await;
                            let mut placeholder_info = Vec::new();
                            for (lang_code, lang_name) in
                                lang_codes.into_iter().zip(lang_names.into_iter())
                            {
                                let format_lang_code_and_name =
                                    format!("{}: {}", lang_name,lang_code);
                                placeholder_info.push(format_lang_code_and_name);
                            }
                            let get_bucket_lists = s3_ops.get_buckets().await;
                            let existing_buckets = format!(
                                "These buckets are already in your account:\n{:#?}\n",
                                get_bucket_lists
                            );
                            let input_s3_uri = Text::new("Specify the bucket path prefix where all documents of the selected format are stored\n")
                           .with_placeholder(&existing_buckets)
                           .with_initial_value("s3://your_bucket_name/folder_name_which_contains_multiple_document_files")
                           .with_help_message("Inside the bucket's path prefix folder, you can have nested subfolders and multiple documents of the same type, each with different content")
                           .with_formatter(&|input| {
                               format!("Received Input S3 URI: {}\n", input)
                           })
                           .prompt()
                           .unwrap();
                            let target_lang_codes = Text::new("You can specify up to 10 target language codes.To specify multiple codes, use single space to separate the target language codes\n")
                            .with_placeholder(&placeholder_info.join(" | "))
                            .with_help_message("First, copy the language code from the placeholder, write multiple language codes with spaces somewhere, and then paste them here without quotation marks")
                            .with_formatter(&|input| {
                                format!("Received Target Language Codes: {}\n", input)
                            })
                            .prompt()
                            .unwrap();

                            let output_s3_uri = Text::new("Please provide the output S3 path prefix URL where all translation results will be stored\n")
                           .with_placeholder(&existing_buckets)
                           .with_help_message("It should be in the format ---s3://bucket_name/new_folder_name---")
                           .with_formatter(&|input| {
                               format!("Received Output S3 URI: {}\n", input)
                           })
                           .prompt()
                           .unwrap();
                            let role_arn = Text::new("Please provide the Data Access Role ARN that grants Amazon Translate read access to your S3 input data\n")
                           .with_placeholder("An example of what it should look like is: ---arn:aws:iam::account_id:role/role_name---\n")
                           .with_help_message("Click here to learn more")
                           .with_formatter(&|input| {
                               format!("Received Data Access Role Arn: {}\n", input)
                           })
                           .prompt()
                           .unwrap();
                            match (
                                job_name.is_empty(),
                                target_lang_codes.is_empty(),
                                input_s3_uri.is_empty(),
                                document_type.is_empty(),
                                output_s3_uri.is_empty(),
                                role_arn.is_empty(),
                            ) {
                                (false, false, false, false, false, false) => {
                                    let mut build_target_lang_codes = Vec::new();
                                    for lang_code in target_lang_codes.split(" ") {
                                        build_target_lang_codes.push(lang_code.to_string());
                                    }
                                    translate_ops
                                        .start_text_translation_job(
                                            &job_name,
                                            Some(build_target_lang_codes),
                                            &input_s3_uri,
                                            &document_type,
                                            &output_s3_uri,
                                            &role_arn,
                                        )
                                        .await;
                                }
                                _ => println!(
                                    "{}\n",
                                    "Ensure that no fields are left empty".red().bold()
                                ),
                            }
                        }
                        "Describe Text Translation Job\n" => {
                            println!("");
                            println!("{}\n","If you are unable to access the Job ID at all, then exit this operation with empty input.\nAfterward, execute the ---List Text Translation Jobs--- option, which will provide all the Job information without requiring any input from you".yellow().bold());
                            let job_id = Text::new("To obtain details of the Translation Job, please enter the Job ID\n")
                            .with_placeholder("The Job ID is generated when you execute the ---Start Text Translation Job--- option\n")
                            .with_formatter(&|input| format!("Received Job ID: {}\n", input))
                            .prompt()
                            .unwrap();
                            match job_id.is_empty() {
                                false => {
                                    translate_ops.describe_text_translation_job(&job_id).await;
                                }
                                true => println!(
                                    "{}\n",
                                    "Ensure that no fields are left empty".red().bold()
                                ),
                            }
                        }
                        "List Text Translation Jobs\n" => {
                            translate_ops.list_translation_jobs().await;
                        }
                        "Return to the Main Menu\n" => continue 'main,
                        _ => println!("Never Reach"),
                    }
                }
            }

            "Amazon Transcribe\n" => {
                let transcribe_operations = vec![
                    "Start Transcription Job\n",
                    "Get Transcription Job\n",
                    "Transcription Status\n",
                    "Download Transcription Results\n",
                    "Retrieve the transcript from a JSON file\n",
                    "Return to the Main Menu\n",
                ];
                loop {
                    let transcribe_choices = Select::new(
                        "Select the option to execute the operation\n",
                        transcribe_operations.clone(),
                    )
                    .with_page_size(6)
                    .with_help_message(
                        "Only two of the APIs from the transcription service are being utilized",
                    )
                    .prompt()
                    .unwrap();
                    match transcribe_choices {
                        "Start Transcription Job\n" => {
                            let get_bucket_lists = s3_ops.get_buckets().await;
                            let existing_buckets = format!(
                                "These buckets are already in your account: {:#?}",
                                get_bucket_lists
                            );
                            let bucket_name = Text::new("Please enter the output bucket name, where the task's output is stored upon completion\n")
                                .with_placeholder(
                                    &existing_buckets
                                )
                                .with_help_message("The name must begin with a lowercase letter and should be unique\nAn AWS bucket is a type of object storage designed for storing objects")
                                .with_formatter(&|str| format!("Choosen Bucket Is: {str}"))
                                .prompt()
                                .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    let valid_formats =
                                        "  mp3 |  mp4  |  wav  |  flac  |  ogg  |  amr  | webm  ";
                                    let media_format =
                                        Text::new("Choose the media format of your audio source\n")
                                            .with_placeholder(valid_formats)
                                            .with_formatter(&|str| format!(".....{str}.....\n"))
                                            .prompt()
                                            .unwrap();
                                    let object_names =
                                        s3_ops.retrieve_keys_in_a_bucket(&bucket_name).await;
                                    let available_object_names = format!(
                                        "The object names are in the {bucket_name} bucket and the URL should begin with: s3://{bucket_name}/ \n{}\n",
                                        object_names.join("\n")
                                    );
                                    let format_of_s3_url = format!(
                                        "Add the object key after this path: s3://{bucket_name}/"
                                    );
                                    let initial_value = format!("s3://{bucket_name}/");
                                    let key_audio_name =
                    Text::new("Enter the S3 key that contains the audio content you wish to transcribe\n")
                        .with_placeholder(&available_object_names)
                        .with_initial_value(&initial_value)
                        .with_formatter(&|str| format!(".....{str}.....\n"))
                        .with_help_message(&format_of_s3_url)
                        .prompt()
                        .unwrap();
                                    println!("{}","Make sure to create a unique name for each transcription task, as the file is generated based on the job name".yellow().bold());
                                    println!("{}\n\n"," If the same name is used for the next task, it will overwrite the content in the same bucket, potentially causing the JSON parser to fail".yellow().bold());
                                    let job_name = Text::new("Provide a unique, identifiable job name which will later be used to retrieve the transcription results\n")
                .with_formatter(&|str| format!(".....{str}.....\n"))
                .prompt()
                .unwrap();
                                    match (
                                        media_format.is_empty(),
                                        key_audio_name.is_empty(),
                                        job_name.is_empty(),
                                    ) {
                                        (false, false, false) => {
                                            transcribe_ops
                                                .start_transcribe_task(
                                                    &bucket_name,
                                                    &key_audio_name,
                                                    &media_format,
                                                    &job_name,
                                                )
                                                .await;
                                        }
                                        _ => println!("{}\n", "Fields Can't be empty".red().bold()),
                                    }
                                }
                                true => println!("{}\n", "Bucket Name Can't be emty".red().bold()),
                            }
                        }
                        "Get Transcription Job\n" => {
                            let job_name = Text::new("Please enter the job name to retrieve the results of the transcription task's initiation\n")
                             .with_placeholder("You assigned the job name when initiating the transcription task")
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .prompt()
                            .unwrap();
                            match job_name.is_empty() {
                                false => {
                                    let transcribe_output =
                                        transcribe_ops.get_transcribe_results(&job_name).await;
                                    if let Some(mut output) = transcribe_output {
                                        if let Some(status) = output.job_status() {
                                            match status.as_str() {
                                                "COMPLETED" => {
                                                    println!(
                                                        "{}\n",
                                                        "The job Status is COMPLETED\n"
                                                            .green()
                                                            .bold()
                                                    );
                                                    output.print_transcription_info_as_text();

                                                    println!("{}\n","Alternatively Execute 'Download Transcription Results' to download all the files generated in 'Get Transcription Task' without leaving the Application".yellow().bold());
                                                }

                                                _ => {
                                                    println!("{}","Execute the 'Transcription Status' option to check the status of the transcription task".yellow().bold());
                                                    println!("{} '{}'\n","You will receive the output only when the status is".yellow().bold(),"COMPLETED".green().bold());
                                                }
                                            }
                                        }
                                    }
                                }
                                true => println!("{}\n", "Job name can't be empty".red().bold()),
                            }
                        }
                        "Transcription Status\n" => {
                            let job_name = Text::new(
                                "Please enter the job name to display its status\n",
                            )
                            .with_placeholder(
                                "You assigned the job name when initiating the transcription task",
                            )
                            .with_formatter(&|str| format!(".....{str}.....\n"))
                            .prompt()
                            .unwrap();
                            match job_name.is_empty() {
                                false => {
                                    let transcribe_output =
                                        transcribe_ops.get_transcribe_results(&job_name).await;
                                    if let Some(mut output) = transcribe_output {
                                        if let Some(status) = output.job_status() {
                                            match status.as_str() {
                                                "COMPLETED" => {
                                                    println!(
                                                        "{}\n",
                                                        "The job Status is COMPLETED"
                                                            .green()
                                                            .bold()
                                                    );
                                                    println!("{}\n","Now, you can go ahead and execute the 'Get Transcribe Job' option to obtain the result".green().bold());
                                                }
                                                "QUEUED" => {
                                                    println!(
                                                        "{}\n",
                                                        "The job Status is QUEUED".yellow().bold()
                                                    );
                                                    println!(
                                                        "{}\n",
                                                        "Let's try again after some time"
                                                            .yellow()
                                                            .bold()
                                                    );
                                                }
                                                "IN_PROGRESS" => {
                                                    println!(
                                                        "{}\n",
                                                        "The job Status is IN_PROGRESS"
                                                            .yellow()
                                                            .bold()
                                                    );
                                                    println!(
                                                        "{}\n",
                                                        "Let's try again after some time"
                                                            .yellow()
                                                            .bold()
                                                    );
                                                }
                                                "FAILED" => {
                                                    println!(
                                                        "{}\n",
                                                        "The job Status is FAILED".yellow().bold()
                                                    );
                                                    println!(
                                                        "Failed Reason: {}\n",
                                                        output
                                                            .failure_reason()
                                                            .unwrap_or(
                                                                "No Failure Reason Is Available"
                                                                    .into()
                                                            )
                                                            .yellow()
                                                            .bold()
                                                    );
                                                }

                                                _ => println!("This can't be reached"),
                                            }
                                        }
                                    }
                                }
                                true => println!("{}\n", "Job name can't be empty".red().bold()),
                            }
                        }
                        "Download Transcription Results\n" => {
                            println!("");
                            println!("{}\n\n","Ensure that no 'TranscribeOutputs' directory exists where the binary is running".yellow().bold());
                            match std::fs::remove_dir_all("TranscribeOutputs/"){
                                Ok(_) => println!("{}\n","TranscribeOutputs dir in the current directory have been deleted.".red().bold()),
                                Err(_) => println!("{}\n","Sure no TranscribeOutputs directory exist in the current directory".yellow().bold())
                            };
                            let get_buckets = s3_ops.get_buckets().await;
                            let available_buckets =
                                format!("Available buckets in your account:\n{:#?}\n", get_buckets);
                            let bucket_name = Text::new("Please enter the bucket name where the 'Start Transcription Job' was initiated\n")
                         .with_placeholder(&available_buckets)
                         .with_formatter(&|str| format!(".....{str}.....\n"))
                         .with_help_message("The bucket name should be the same as where the 'Start Transcription Job' was initiated,\n  as the key path is used to download the content without requiring manual input")
                         .prompt()
                         .unwrap();
                            match bucket_name.is_empty() {
                                false => {
                                    s3_ops.download_transcription_results(&bucket_name).await;
                                }
                                true => println!("{}\n", "Bucket Name can't be empty".red().bold()),
                            }
                        }
                        "Retrieve the transcript from a JSON file\n" => {
                            let json_path = Text::new("Please provide the path to the JSON file you downloaded either from 'Download Transcription Results' or manually from the web console\n")
                                             .with_placeholder("Do not pass any JSON data; this is meant to parse data specific to the transcript JSON file\n")
                                             .prompt()
                                             .unwrap();
                            match json_path.is_empty() {
                                false => {
                                    let json_data = read_to_string(&json_path)
                                        .expect("Error while opening the path you specified");

                                    let parsed_data: Value = serde_json::from_str(&json_data)
                                        .expect("Failed to parse JSON");
                                    let transcripts: &Map<String, Value> =
                                        parsed_data["results"].as_object().unwrap();
                                    if let Some(transcript) = transcripts["transcripts"].as_array()
                                    {
                                        let mut file = File::create("transcript.txt")
                                            .expect("Error while creating file\n");
                                        transcript.iter().for_each(|data| {
                                            let transcript = data["transcript"]
                                                .as_str()
                                                .expect("The transcript key shoud exist\n");
                                            match file.write_all(transcript.as_bytes()){
                                                  Ok(_) => println!("The transcript has been successfully written to the current directory with the name '{}'","transcript.txt".green().bold()),
                                                  Err(_) => println!("")
                                            }
                                        });
                                    }
                                }
                                true => {}
                            }
                        }
                        "Return to the Main Menu\n" => continue 'main,
                        _ => println!("Never Reach"),
                    }
                }
            }
            "Quit the application\n" => {
                credential.empty();
                break 'main;
            }
            _other => {
                println!("This branch never reach..");
            }
        }
    }
}
fn global_render_config() -> RenderConfig {
    let mut config = RenderConfig::default()
        .with_prompt_prefix(Styled::new("⚙️").with_fg(inquire::ui::Color::DarkBlue))
        .with_text_input(StyleSheet::new().with_fg(inquire::ui::Color::LightGreen))
        .with_highlighted_option_prefix(Styled::new("👉"))
        .with_help_message(StyleSheet::new().with_fg(inquire::ui::Color::DarkYellow));
    config.answer = StyleSheet::new()
        .with_attr(Attributes::BOLD)
        .with_fg(inquire::ui::Color::DarkGreen);
    config
}
