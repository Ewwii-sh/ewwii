use regex::Regex;
use std::collections::HashSet;

pub fn extract_poll_and_listen_vars(code: &str) -> HashSet<String> {
    let mut vars = HashSet::new();

    let re = Regex::new(r#"\b(poll|listen)\s*\(\s*['"]([^'"]+)['"]"#).unwrap();

    for cap in re.captures_iter(code) {
        let var_name = &cap[2];
        vars.insert(var_name.to_string());
    }

    vars
}

#[cfg(test)]
mod tests {
    use crate::helper::extract_poll_and_listen_vars;
    #[test]
    fn poll_listen_regex_test() {
        let result = extract_poll_and_listen_vars(
            r#"
            fn widget1() {
                return box(#{
                    class: 'widget1',
                    orientation: 'h',
                    space_evenly: true,
                    halign: 'start',
                    spacing: 5
                }, [
                label(#{ text: 'Hello Ewwii!' }),
                slider(#{ min: 0, max: 101, value: 3, onchange: 'echo hi' }), 
                button(#{ onclick: 'notify-send 'hello there!'', label: 'greet' }),
                label(#{ text: cpu_usage }),
                ]);
            };

            enter([
                poll('cpu_usage', #{ 
                    interval: '1s', 
                    cmd: 'echo hi',  
                    initial: 'initial' 
                }),
                listen('net_speed', #{ 
                    cmd: 'while true; do date +%T; sleep 1; done'
                }),

                defwindow('main_window', #{
                    monitor: 0,
                    windowtype: 'dock',
                    geometry: #{ x: '0px', y: '0px', width: '10px', height: '20px' },
                }, widget1())
            ]);

    "#,
        );
        println!("{:#?}", result);
        assert!(result.contains("cpu_usage"));
        assert!(result.contains("net_speed"));
    }
}
