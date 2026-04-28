/// converts normal page name with spaces and other things into the expected wikipedia notation
/// not everything is handled!!!
pub fn convert_to_wikipedia_page_name_convention(input: &str) -> String {
	// spaces become _
	let mut s = input.replace(' ', "_");

	// make the first character uppercase
	if let Some(first_char) = s.chars().next() {
		let upper = first_char.to_uppercase().to_string();
		s.replace_range(0..first_char.len_utf8(), &upper);
	}

	s
}
