# leptos-locatorjs

EARLY DEVELOPMENT STAGE. DO NOT EXPECT THIS TO WORK.

This macro adds the data-locatorjs-id attribute to all div, h1, h2, etc. elements in the Rust source code. The attribute is used by the LocatorJS library to uniquely identify HTML elements in automated tests.

Example:
```rust
#[component]
#[leptos_locatorjs::add_locatorjs_id]
pub fn Example() -> impl IntoView {
    let (count, _) = create_signal(2);
    let ressource = create_resource(|| (), |_| async move { pray_me().await });

    let hello_word = move || {
        let my_count = count.get();
        match my_count {
            2 => view! {<div>"Hello, world!"</div>},
            _ => view! {<div>"Burn, world!"</div>},
        }
    };

    let god____where_r_u = move || {
        let _son_______i_am_everywhere = ressource.get();
        "Je suis là, mon fils"
    };

    view! {
        <div>
            <h1>{hello_word}</h1>
            <Suspense fallback=|| view!{ <div>"Loading..."</div> }>
                <ul>
                    <li>"I like banana."</li>
                    <li>{god____where_r_u}</li>
                </ul>
            </Suspense>
        </div>
    }
}
```
