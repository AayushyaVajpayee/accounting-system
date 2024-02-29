#import "@preview/tablex:0.0.8": tablex,colspanx, cellx,vlinex,hlinex
#let spread_lines(invoice_lines,headers)={
  for l in invoice_lines{
    headers.map(h=>l.at(h))
  }
}
#let make_bold(hdr)={
  [*#hdr*]
}

#let invoice_line_tableV2(invoice_lines)=[
  #align(center,tablex(
    columns:invoice_lines.header_and_units.len(),
    auto-hlines: false,
    align:center+horizon,
    header-rows:2,
    fill:(col, _r) => if calc.even(_r) and _r!=0 { luma(240) } else { white },
    hlinex(),
    ..invoice_lines.header_and_units.map(it=>it.first()).
    map(it=>[*#it*]),
    ..invoice_lines.header_and_units.map(it=>it.at(1)).
    map(it=>it),
    hlinex(),
    ..spread_lines(invoice_lines.lines,invoice_lines.header_and_units.map(it=>it.at(2))),
    hlinex(),
    colspanx(invoice_lines.header_and_units.len()-1)[#align(right,[*total amount* (add additional ..charge breakup above)])],[193424.00],
     hlinex()
  ))
]

