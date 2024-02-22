#import "@preview/tablex:0.0.8": tablex, cellx,vlinex,hlinex
#import "invoice_lines.typ"
#import "tax_summary.typ"
#import "invoice_summary.typ"
#set page(flipped: true)
#let invoice_model = json("invoice_data.json")
#let supplier_heading(name,gstin,address)=[
 #grid(columns: (1fr,3fr,1fr),
 align(center+horizon)[#image("sunset.png")],
   align(center+horizon, text(12pt)[
  = *#name*
    #address
  ])

 ,figure(image("einvoice_qr.png"),caption:[einvoicing qr code],numbering:none)
)

]


#let header_details(supplier,billed_to,shipped_to)=[
  #tablex(
    auto-vlines: false,
    columns: (0.4fr,1fr,1fr,1fr),
    fill:(col, _r) => if calc.odd(_r) { luma(240) } else { white },
    align:(col, row) =>
    if row == 0 { center }
    else if col == 0 { left+horizon }
    else { right },
    auto-hlines:false,
    vlinex(),(),(),(),vlinex(),
    hlinex(),
    [],[*supplier*],[*billed to*],[*shipped to*],
    [*name*],supplier.name,billed_to.name,shipped_to.name,
    [*gstin*],supplier.gstin,billed_to.gstin,shipped_to.gstin,
    [*address*],supplier.address,billed_to.address,shipped_to.address,hlinex()
  )
]
#let prepare_header_key_vals(hdrs)=[
  #set terms(separator: [: ])
  / Invoice no:#hdrs.invoice_number

  / Invoice date:#datetime(year:hdrs.invoice_date.year,
month:hdrs.invoice_date.month,
day:hdrs.invoice_date.day).display("[day]-[month repr:short]-[year]")

  / Order no:#hdrs.order_number

  / Order date:#datetime(
    year:hdrs.order_date.year,
    month:hdrs.order_date.month,
    day:hdrs.order_date.day
  ).display("[day]-[month repr:short]-[year]")

  / Payment terms: #hdrs.payment_terms

  / IRN no: #hdrs.irn_no

]
#show: set page(margin: (x:10pt,y:5pt))
#supplier_heading(invoice_model.supplier.name,invoice_model.supplier.gstin,invoice_model.supplier.address)
#line(length: 100%)

#grid(columns: (2.8fr,0.05fr,1.15fr),
header_details(invoice_model.supplier,
invoice_model.billed_to,
invoice_model.shipped_to),
[],
prepare_header_key_vals(invoice_model)
)
#invoice_lines.invoice_line_tableV2(invoice_model.invoice_lines_table)

#grid(

  columns:(1fr,0.5fr,1fr),
  align(center,tax_summary.tax_summary_table(invoice_model.tax_summary)),[],
  align(center,invoice_summary.invoice_summary(invoice_model.invoice_summary))
)
