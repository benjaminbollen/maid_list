extern crate hyper;
extern crate glib;
extern crate gtk;
extern crate rustc_serialize;
extern crate iron;
extern crate router;

use gtk::traits::*;
use gtk::signal::Inhibit;
use gtk::widgets::Builder;
use gtk::Window;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

use rustc_serialize::{Decoder};
use rustc_serialize::json::{self};

use hyper::Client;
use hyper::header::Connection;

use iron::prelude::*;

#[derive(RustcDecodable, RustcEncodable)]
struct SingleData {
    balance: i32,
    reserved_balance: i32,
    address: String,
}

fn append_column(title: &str, v: &mut Vec<gtk::TreeViewColumn>) {
    let l = v.len();
    let renderer = gtk::CellRendererText::new().unwrap();

    v.push(gtk::TreeViewColumn::new().unwrap());
    let tmp = v.get_mut(l).unwrap();

    tmp.set_title(title);
    tmp.set_resizable(true);
    tmp.pack_start(&renderer, true);
    tmp.add_attribute(&renderer, "text", l as i32);
}


fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let glade_src = include_str!("maidlist.glade");
    let builder = Builder::new_from_string(glade_src).unwrap();  

    let sortbutton: gtk::Button = builder.get_object("button1").unwrap();
    let sortentry: gtk::Entry = builder.get_object("entry1").unwrap();

    let balance_entry: gtk::Entry = builder.get_object("balanceEntry").unwrap();
    let balance_label: gtk::Label = builder.get_object("balanceLabel").unwrap();
    let balance_button: gtk::Button = builder.get_object("balanceButton").unwrap();

    let clone_send_address_entry = sortentry.clone();

    
    let window: Window = builder.get_object("window1").unwrap();
    //let search_button: gtk::Button = builder.get_object("name").unwrap();
    window.set_default_size(600, 420);

    window.set_window_position(gtk::WindowPosition::Center);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });

    // test Value
    let mut value = glib::Value::new();
    let mut value2 = glib::Value::new();

    value.init(glib::Type::String);
    value2.init(glib::Type::ISize);

     // left pane

    let left_tree: gtk::TreeView = builder.get_object("treeview4").unwrap();
    let another_tree = left_tree.clone();
    let another_tree2 = left_tree.clone();

    let left_store = gtk::ListStore::new(&[glib::Type::String, glib::Type::USize]).unwrap();
    let left_model = left_store.get_model().unwrap();
    let left_model1 = left_model.clone();
    let left_selection = another_tree.get_selection().unwrap();

    let left_storecell = Rc::new(RefCell::new(left_store));
    let left_storecell_clone = left_storecell.clone();

    let mut columns : Vec<gtk::TreeViewColumn> = Vec::new();

    append_column("address", &mut columns);
    append_column("balance", &mut columns);

    for i in columns {
            another_tree2.append_column(&i);
    }
    balance_button.connect_clicked(move |_| {
        let client = Client::new();

        let the_address = balance_entry.get_text().unwrap();
        let mut results = client.get("https://www.omniwallet.org/v1/mastercoin_verify/addresses?currency_id=3")
        .header(Connection::close())
        .send().unwrap();
        let mut payload = String::new();
        results.read_to_string(&mut payload).unwrap();
        let decoded: Vec<SingleData> = json::decode(&payload).unwrap();
        for thethings in 0..decoded.len() {
            if decoded[thethings].address == the_address {
                balance_label.set_text(&decoded[thethings].balance.to_string());
            }
        }
    });

    sortbutton.connect_clicked(move |_| {
        let the_text= clone_send_address_entry.get_text().unwrap();
        let the_number: i32 = the_text.parse().unwrap();
        let client = Client::new();

        let mut results = client.get("https://www.omniwallet.org/v1/mastercoin_verify/addresses?currency_id=3")
        .header(Connection::close())
        .send().unwrap();
        let mut payload = String::new();
        results.read_to_string(&mut payload).unwrap();
        left_storecell_clone.borrow_mut().clear();
        let mut ints = 0;
        let decoded: Vec<SingleData> = json::decode(&payload).unwrap();
        for thethings in 0..decoded.len() {
            if decoded[thethings].balance > the_number {
                let mut top_level = gtk::TreeIter::new();

                let mut val1 = glib::Value::new();
                val1.init(glib::Type::ISize);
                val1.set_long(decoded[thethings].balance as i64);
                left_storecell_clone.borrow_mut().append(&mut top_level);
                left_storecell_clone.borrow_mut().set_string(&top_level, 0, &decoded[thethings].address);
                left_storecell_clone.borrow_mut().set_value(&top_level, 1, &val1);
                ints += 1;
            }
        }
            println!("number of addresses in query: {:?}", ints);
        let the_real_left_model = left_storecell_clone.borrow().get_model().unwrap();

        left_tree.set_model(&the_real_left_model);
        left_tree.set_headers_visible(false);
    });


    // print out when a row is selected

    left_selection.connect_changed(move |tree_selection| {
        let mut iter = gtk::TreeIter::new();
        tree_selection.get_selected(&left_model1, &mut iter);
        if let Some(path) = left_model1.get_path(&iter) {
            println!("selected row {}", path.to_string().unwrap());
        }
    });

    window.show_all();
    gtk::main();

}

